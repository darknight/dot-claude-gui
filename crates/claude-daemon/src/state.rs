use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use claude_config::parse::read_settings;
use claude_types::{Settings, WsEvent};
use tokio::sync::{RwLock, broadcast};

/// Information about a registered project.
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
}

/// The inner state shared across all request handlers.
pub struct AppStateInner {
    pub claude_home: PathBuf,
    pub user_settings: RwLock<Settings>,
    pub project_settings: RwLock<HashMap<String, Settings>>,
    pub local_settings: RwLock<HashMap<String, Settings>>,
    pub projects: RwLock<Vec<ProjectInfo>>,
    pub auth_token: String,
    pub ws_tx: broadcast::Sender<WsEvent>,
    pub started_at: std::time::Instant,
}

/// Arc-wrapped state, cheap to clone across handlers.
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    /// Create a new `AppState` rooted at `claude_home` and protected by `auth_token`.
    pub fn new(claude_home: PathBuf, auth_token: String) -> Self {
        let (ws_tx, _) = broadcast::channel(256);
        Self {
            inner: Arc::new(AppStateInner {
                claude_home,
                user_settings: RwLock::new(Settings::default()),
                project_settings: RwLock::new(HashMap::new()),
                local_settings: RwLock::new(HashMap::new()),
                projects: RwLock::new(Vec::new()),
                auth_token,
                ws_tx,
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

    /// Broadcast a `WsEvent` to all active WebSocket subscribers.
    pub fn broadcast(&self, event: WsEvent) {
        // Ignore error when there are no subscribers.
        let _ = self.inner.ws_tx.send(event);
    }
}
