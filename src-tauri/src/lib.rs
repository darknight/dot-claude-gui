mod app_config;
mod commands;
mod events;
mod executor;
mod state;
mod watcher;

use std::path::PathBuf;
use tauri::Manager;

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

// ── IPC commands ──────────────────────────────────────────────────────────────

#[tauri::command]
fn get_config_dir() -> Result<String, String> {
    config_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn read_app_config() -> Result<String, String> {
    let path = ensure_config_dir()?.join("config.json");
    let cfg = app_config::read_config(&path)?;
    serde_json::to_string(&cfg).map_err(|e| format!("serialize app config: {e}"))
}

#[tauri::command]
fn write_app_config(json: String) -> Result<(), String> {
    let cfg: app_config::AppConfig = serde_json::from_str(&json)
        .map_err(|e| format!("parse app config: {e}"))?;
    let path = ensure_config_dir()?.join("config.json");
    app_config::write_config(&path, &cfg)
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_config_dir,
            read_app_config,
            write_app_config,
            commands::health::health,
            commands::config::get_user_config,
            commands::config::update_user_config,
            commands::gui_projects::gui_list_projects,
            commands::gui_projects::add_project,
            commands::gui_projects::bind_project,
            commands::gui_projects::unbind_project,
            commands::gui_projects::remove_project,
            commands::gui_projects::update_project_launch,
            commands::gui_projects::update_project_path,
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
            commands::launcher::get_claude_args,
            commands::accounts::list_accounts,
            commands::accounts::create_account,
            commands::accounts::delete_account,
            commands::accounts::get_account_status,
            commands::account_session::set_active_account,
            commands::account_session::account_overview,
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
            commands::project_facets::project_read_settings,
            commands::project_facets::project_write_settings,
            commands::project_facets::project_read_claudemd,
            commands::project_facets::project_write_claudemd,
            commands::project_facets::project_list_memory,
            commands::project_facets::project_read_memory_file,
            commands::project_facets::project_write_memory_file,
            commands::project_facets::project_delete_memory_file,
            commands::project_facets::project_list_plugins,
            commands::project_facets::project_read_effective,
            commands::migration::take_migration_report,
        ])
        .setup(|app| {
            // One-shot migration v1 → v2 (idempotent for v2).
            // Runs before any state init so subsequent code reads the new schema.
            // Report is stashed in AppState and pulled via IPC (not emitted as an
            // event) to avoid the setup-vs-mount race: setup runs before the WebView
            // window / JS bundle exist, so events emitted here are lost.
            let pending_report: Option<app_config::MigrationReport> =
                if let Ok(dir) = ensure_config_dir() {
                    let cfg_path = dir.join("config.json");
                    let native_exists = dirs_next::home_dir()
                        .map(|h| h.join(".claude").exists())
                        .unwrap_or(false);
                    match app_config::migrate_at_startup(&cfg_path, native_exists) {
                        Ok(report) => {
                            tracing::info!("config migration: {report:?}");
                            Some(report)
                        }
                        Err(e) => {
                            tracing::error!("config migration failed: {e}");
                            None
                        }
                    }
                } else {
                    None
                };

            let claude_home = dirs_next::home_dir()
                .ok_or_else(|| "cannot determine home directory".to_string())?
                .join(".claude");
            let app_state = crate::state::AppState::new(claude_home);
            app.manage(app_state);

            // Load initial user settings before the watcher starts.
            let state_handle = app.state::<crate::state::AppState>();
            let state_clone = (*state_handle).clone();
            tauri::async_runtime::block_on(async {
                if let Err(e) = state_clone.load_user_settings().await {
                    tracing::warn!("failed to load initial user settings: {e}");
                }
                // Stash migration report (if any) now that AppState is ready.
                if let Some(report) = pending_report {
                    state_clone.set_migration_report(report).await;
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
