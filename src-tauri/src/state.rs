use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use claude_config::parse::read_settings;
use claude_types::Settings;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Information about a registered project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
}

/// The inner state shared across all Tauri command handlers.
pub struct AppStateInner {
    pub claude_home: PathBuf,
    pub projects_file: Option<PathBuf>,
    pub user_settings: RwLock<Settings>,
    pub project_settings: RwLock<HashMap<String, Settings>>,
    pub local_settings: RwLock<HashMap<String, Settings>>,
    pub projects: RwLock<Vec<ProjectInfo>>,
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
        Self::with_projects_file(claude_home, None)
    }

    /// Create a new `AppState` with a backing file for the project registry.
    pub fn with_projects_file(claude_home: PathBuf, projects_file: Option<PathBuf>) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                claude_home,
                projects_file,
                user_settings: RwLock::new(Settings::default()),
                project_settings: RwLock::new(HashMap::new()),
                local_settings: RwLock::new(HashMap::new()),
                projects: RwLock::new(Vec::new()),
                started_at: std::time::Instant::now(),
            }),
        }
    }

    /// Read user settings from disk and populate the cache.
    pub async fn load_user_settings(&self) -> Result<()> {
        let settings_path = self.inner.claude_home.join("settings.json");
        let settings = read_settings(&settings_path)?;
        *self.inner.user_settings.write().await = settings;
        Ok(())
    }

    /// Load the project registry from disk into state.
    /// No-op if no projects_file is configured or the file doesn't exist.
    pub async fn load_projects(&self) -> Result<()> {
        let Some(path) = self.inner.projects_file.clone() else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }
        let contents = std::fs::read_to_string(&path)?;
        let projects: Vec<ProjectInfo> = serde_json::from_str(&contents)?;
        *self.inner.projects.write().await = projects;
        Ok(())
    }

    /// Persist the current project registry to disk.
    /// No-op if no projects_file is configured.
    pub async fn save_projects(&self) -> Result<()> {
        let Some(path) = self.inner.projects_file.clone() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let projects = self.inner.projects.read().await;
        let json = serde_json::to_string_pretty(&*projects)?;
        std::fs::write(&path, json)?;
        Ok(())
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
        assert!(state.inner.project_settings.read().await.is_empty());
        assert!(state.inner.projects.read().await.is_empty());
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
