use std::path::Path;

use anyhow::Result;
use claude_types::{WsEvent, WsValidationError};
use tracing::{debug, warn};

use crate::state::AppState;

/// Start the file watcher on a background thread, bridging sync fs events to
/// the async tokio runtime for state updates and WebSocket broadcasts.
///
/// The returned `Result` only covers watcher initialisation; ongoing errors are
/// logged and broadcast as `WsEvent::ValidationError`.
pub fn start_watcher(state: AppState) -> Result<()> {
    let claude_home = state.inner.claude_home.clone();
    let paths = vec![claude_home.clone()];
    let (_watcher, rx) = claude_config::watch::watch_directories(&paths)?;

    // Spawn a dedicated OS thread to receive synchronous mpsc events from
    // `notify`. We then bridge each event to the tokio runtime via
    // `Handle::current()` (valid because `start_watcher` is called from an
    // async context).
    let handle = tokio::runtime::Handle::current();

    std::thread::spawn(move || {
        // Keep `_watcher` alive for the lifetime of this thread so the OS-level
        // watches are not dropped.
        let _watcher = _watcher;

        loop {
            match rx.recv() {
                Ok(event) => {
                    let state = state.clone();
                    let claude_home = claude_home.clone();
                    handle.spawn(async move {
                        handle_file_event(&state, &claude_home, &event.path).await;
                    });
                }
                Err(_) => {
                    debug!("file watcher channel closed, watcher thread exiting");
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Determine what changed and update state / broadcast the appropriate event.
async fn handle_file_event(state: &AppState, claude_home: &Path, changed: &Path) {
    // ── User settings ────────────────────────────────────────────────────────
    let user_settings_path = claude_home.join("settings.json");
    if changed == user_settings_path {
        debug!("user settings changed: {}", changed.display());
        match claude_config::parse::read_settings(&user_settings_path) {
            Ok(new_settings) => {
                *state.inner.user_settings.write().await = new_settings.clone();
                state.broadcast(WsEvent::ConfigChanged {
                    settings: new_settings,
                    source: Some("file-watcher".to_string()),
                });
            }
            Err(e) => {
                warn!("failed to parse user settings: {e}");
                broadcast_parse_error(state, &user_settings_path, &e.to_string());
            }
        }
        return;
    }

    // ── Project settings ─────────────────────────────────────────────────────
    // Check registered projects whose `.claude/settings.json` or
    // `.claude/settings.local.json` matches the changed path.
    let projects = state.inner.projects.read().await;
    for project in projects.iter() {
        let dot_claude = project.path.join(".claude");
        let project_settings_path = dot_claude.join("settings.json");
        let local_settings_path = dot_claude.join("settings.local.json");

        if changed == project_settings_path || changed == local_settings_path {
            let is_local = changed == local_settings_path;
            debug!(
                "project {} settings changed (local={}): {}",
                project.id,
                is_local,
                changed.display()
            );

            match claude_config::parse::read_settings(changed) {
                Ok(new_settings) => {
                    if is_local {
                        state
                            .inner
                            .local_settings
                            .write()
                            .await
                            .insert(project.id.clone(), new_settings.clone());
                    } else {
                        state
                            .inner
                            .project_settings
                            .write()
                            .await
                            .insert(project.id.clone(), new_settings.clone());
                    }
                    state.broadcast(WsEvent::ConfigChanged {
                        settings: new_settings,
                        source: Some("file-watcher".to_string()),
                    });
                }
                Err(e) => {
                    warn!("failed to parse project settings {}: {e}", changed.display());
                    broadcast_parse_error(state, changed, &e.to_string());
                }
            }
            return;
        }
    }
}

/// Broadcast a `ValidationError` event with a single parse error.
fn broadcast_parse_error(state: &AppState, path: &Path, message: &str) {
    state.broadcast(WsEvent::ValidationError {
        errors: vec![WsValidationError {
            field: path.display().to_string(),
            message: message.to_string(),
        }],
    });
}
