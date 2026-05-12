use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use claude_config::parse::read_settings;
use claude_types::Settings;
use tokio::sync::RwLock;

/// The inner state shared across all Tauri command handlers.
pub struct AppStateInner {
    pub claude_home: PathBuf,
    /// Currently-targeted account directory. Defaults to `claude_home`
    /// (native ~/.claude/). Mutated via `commands::account_session::set_active_account`.
    pub active_account_dir: RwLock<PathBuf>,
    pub user_settings: RwLock<Settings>,
    pub started_at: std::time::Instant,
}

/// Arc-wrapped state, cheap to clone across Tauri commands.
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    /// Create a new `AppState` rooted at `claude_home`.
    pub fn new(claude_home: PathBuf) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                active_account_dir: RwLock::new(claude_home.clone()),
                claude_home,
                user_settings: RwLock::new(Settings::default()),
                started_at: std::time::Instant::now(),
            }),
        }
    }

    /// Snapshot of the currently-active account dir. All IPC handlers that
    /// read user-layer files (settings, plugins, skills, CLAUDE.md, memory,
    /// MCP) should go through this rather than `inner.claude_home`.
    pub async fn current_dir(&self) -> PathBuf {
        self.inner.active_account_dir.read().await.clone()
    }

    /// Swap the active account dir. Caller is responsible for downstream
    /// invalidation (cache reload + watcher restart) — see
    /// `commands::account_session::set_active_account`.
    pub async fn set_active_account_dir(&self, dir: PathBuf) {
        *self.inner.active_account_dir.write().await = dir;
    }

    /// Read user settings from disk and populate the cache.
    pub async fn load_user_settings(&self) -> Result<()> {
        let dir = self.current_dir().await;
        let settings_path = dir.join("settings.json");
        let settings = read_settings(&settings_path)?;
        *self.inner.user_settings.write().await = settings;
        Ok(())
    }

    /// Given a project path (absolute), look up its account binding in
    /// `~/.dot-claude-gui/config.json` and return the resolved account directory.
    ///
    /// - Unbound project → `Err("Unbound project: …")`
    /// - Unknown account name → `Err("Unknown account: …")`
    /// - `"default"` binding → `<home>/.claude/`
    /// - named account → `<home>/.dot-claude-gui/accounts/<name>/`
    pub async fn resolve_project_account_dir(&self, project_path: &str) -> Result<PathBuf, String> {
        let home = dirs_next::home_dir()
            .ok_or_else(|| "cannot determine home directory".to_string())?;
        let cfg_path = home.join(".dot-claude-gui").join("config.json");
        let cfg = crate::app_config::read_config(&cfg_path)?;
        crate::app_config::resolve_account_dir_for_project(&home, &cfg, project_path)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn app_state_new_starts_empty() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        assert_eq!(state.inner.claude_home, dir.path());
        assert!(state.inner.user_settings.read().await.env.is_none());
    }

    #[tokio::test]
    async fn app_state_loads_user_settings_from_disk() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        std::fs::write(
            &settings_path,
            r#"{"env": {"FOO": "bar"}}"#,
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.load_user_settings().await.unwrap();

        let loaded = state.inner.user_settings.read().await;
        // Settings.env is Option<HashMap<String, String>>, so unwrap the Option first.
        assert_eq!(
            loaded.env.as_ref().and_then(|m| m.get("FOO")).map(String::as_str),
            Some("bar")
        );
    }
}
