mod commands;
mod events;
mod executor;
mod state;
mod watcher;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: String,
    pub language: String,
    pub font_size: u32,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    #[serde(default = "default_subpanel_width")]
    pub subpanel_width: u32,
}

fn default_sidebar_width() -> u32 { 56 }
fn default_subpanel_width() -> u32 { 240 }

// ── Config-dir helpers ────────────────────────────────────────────────────────

fn config_dir() -> Result<PathBuf, String> {
    let home = dirs_next::home_dir().ok_or("cannot determine home directory")?;
    Ok(home.join(".dot-claude-gui"))
}

fn ensure_config_dir() -> Result<PathBuf, String> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create config dir: {}", e))?;
    Ok(dir)
}

fn default_app_config() -> AppConfig {
    AppConfig {
        theme: "system".to_string(),
        language: "zh-CN".to_string(),
        font_size: 14,
        sidebar_width: 56,
        subpanel_width: 240,
    }
}

// ── IPC commands ──────────────────────────────────────────────────────────────

#[tauri::command]
fn get_config_dir() -> Result<String, String> {
    config_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn read_app_config() -> Result<String, String> {
    let dir = ensure_config_dir()?;
    let path = dir.join("config.json");
    if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read config.json: {}", e))
    } else {
        let defaults = default_app_config();
        serde_json::to_string(&defaults)
            .map_err(|e| format!("failed to serialize default app config: {}", e))
    }
}

#[tauri::command]
fn write_app_config(json: String) -> Result<(), String> {
    let dir = ensure_config_dir()?;
    let path = dir.join("config.json");
    std::fs::write(&path, json)
        .map_err(|e| format!("failed to write config.json: {}", e))
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_config_dir,
            read_app_config,
            write_app_config,
            commands::health::health,
            commands::config::get_user_config,
            commands::config::update_user_config,
            commands::config::get_project_config,
            commands::config::update_project_config,
            commands::config::get_effective_config,
            commands::projects::list_projects,
            commands::projects::register_project,
            commands::projects::unregister_project,
            commands::skills::list_skills,
            commands::skills::get_skill_content,
            commands::claudemd::list_claudemd_files,
            commands::claudemd::get_claudemd_file,
            commands::claudemd::update_claudemd_file,
            commands::claudemd::delete_claudemd_file,
            commands::memory::list_memory_projects,
            commands::memory::list_memory_files,
            commands::memory::get_memory_file,
            commands::memory::update_memory_file,
            commands::memory::delete_memory_file,
            commands::launcher::launch_claude,
            commands::mcp::list_mcp_servers,
            commands::mcp::add_mcp_server,
            commands::mcp::remove_mcp_server,
            commands::plugins::list_plugins,
            commands::plugins::list_marketplaces,
            commands::plugins::get_marketplace_plugins,
            commands::plugins::toggle_plugin,
            commands::plugins::install_plugin,
            commands::plugins::uninstall_plugin,
            commands::plugins::add_marketplace,
            commands::plugins::remove_marketplace,
        ])
        .setup(|app| {
            let claude_home = dirs_next::home_dir()
                .ok_or_else(|| "cannot determine home directory".to_string())?
                .join(".claude");
            let app_state = crate::state::AppState::new(claude_home);
            app.manage(app_state);

            // Load initial user settings into the cache before the watcher starts.
            let state_handle = app.state::<crate::state::AppState>();
            let state_clone = (*state_handle).clone();
            tauri::async_runtime::block_on(async {
                if let Err(e) = state_clone.load_user_settings().await {
                    tracing::warn!("failed to load initial user settings: {e}");
                }
            });

            // Start the file watcher inside the async runtime so that
            // tokio::runtime::Handle::current() inside start_watcher succeeds.
            // Tauri's setup() is synchronous and has no tokio context, but
            // tauri::async_runtime::spawn runs on the global tokio runtime.
            let app_handle_for_watcher = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                crate::watcher::start_watcher(app_handle_for_watcher, state_clone);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
