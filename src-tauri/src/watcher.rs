use std::path::Path;

use tauri::{AppHandle, Emitter};
use tracing::{debug, warn};

use crate::events::{
    ConfigChangedPayload, ValidationErrorPayload, EVT_CONFIG_CHANGED, EVT_VALIDATION_ERROR,
};
use crate::state::AppState;

/// Start the file watcher in a background thread. Watches `state.claude_home`
/// and emits `config-changed` / `validation-error` Tauri events on changes.
///
/// The watcher pattern mirrors the daemon: `watch_directories` returns a
/// `(RecommendedWatcher, Receiver<FileChangeEvent>)` pair.  We keep the
/// watcher alive inside the spawned OS thread and bridge events to the tokio
/// runtime via `Handle::current()` (valid because `start_watcher` is called
/// from an async-capable context).
pub fn start_watcher(app: AppHandle, state: AppState) {
    let claude_home = state.inner.claude_home.clone();
    let mut paths = vec![claude_home.clone()];

    // Also watch the GUI accounts root so events fire for any account's
    // user-layer files (Stage 2: account-scoped settings live under
    // ~/.dot-claude-gui/accounts/<name>/).
    if let Some(home) = dirs_next::home_dir() {
        let gui_accounts = home.join(".dot-claude-gui").join("accounts");
        // Create unconditionally so first-account creation fires events for
        // the existing watch (otherwise the watcher would miss the mkdir).
        let _ = std::fs::create_dir_all(&gui_accounts);
        paths.push(gui_accounts);
    }

    // Capture the current tokio runtime handle before spawning the OS thread.
    // This is safe because setup runs inside Tauri's async runtime.
    let handle = tokio::runtime::Handle::current();

    std::thread::spawn(move || {
        let watch_result = claude_config::watch::watch_directories(&paths);
        let (_watcher, rx) = match watch_result {
            Ok(pair) => pair,
            Err(e) => {
                tracing::error!("failed to initialise file watcher: {e}");
                return;
            }
        };

        // Keep `_watcher` alive so OS-level watches aren't dropped.
        loop {
            match rx.recv() {
                Ok(event) => {
                    let app = app.clone();
                    let state = state.clone();
                    handle.spawn(async move {
                        handle_file_event(&app, &state, &event.path).await;
                    });
                }
                Err(_) => {
                    debug!("file watcher channel closed, watcher thread exiting");
                    break;
                }
            }
        }
    });
}

/// Determine what changed and update state / emit the appropriate Tauri event.
async fn handle_file_event(
    app: &AppHandle,
    state: &AppState,
    changed: &Path,
) {
    // ── User settings ────────────────────────────────────────────────────────
    // User-layer settings live in the currently-active account dir (defaults
    // to ~/.claude/, may be ~/.dot-claude-gui/accounts/<name>/).
    let account_dir = state.current_dir().await;
    let user_settings_path = account_dir.join("settings.json");
    if changed == user_settings_path {
        debug!("user settings changed: {}", changed.display());
        match claude_config::parse::read_settings(&user_settings_path) {
            Ok(new_settings) => {
                *state.inner.user_settings.write().await = new_settings.clone();
                let _ = app.emit(
                    EVT_CONFIG_CHANGED,
                    ConfigChangedPayload {
                        settings: new_settings,
                        source: Some("file-watcher".to_string()),
                    },
                );
            }
            Err(e) => {
                warn!("failed to parse user settings: {e}");
                emit_validation_error(app, changed, &e.to_string());
            }
        }
        return;
    }

}

/// Emit a `validation-error` Tauri event with a single parse error.
fn emit_validation_error(app: &AppHandle, path: &Path, message: &str) {
    use claude_types::WsValidationError;
    let _ = app.emit(
        EVT_VALIDATION_ERROR,
        ValidationErrorPayload {
            errors: vec![WsValidationError {
                field: path.display().to_string(),
                message: message.to_string(),
            }],
        },
    );
}
