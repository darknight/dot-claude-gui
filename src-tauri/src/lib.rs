mod commands;
mod events;
mod executor;
mod state;
mod watcher;

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub conn_type: String,
    pub url: String,
    pub token: String,
    pub managed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionsFile {
    pub active_connection_id: String,
    pub connections: Vec<ConnectionEntry>,
}

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

fn default_connections(url: &str, token: &str) -> ConnectionsFile {
    ConnectionsFile {
        connections: vec![ConnectionEntry {
            id: "local".to_string(),
            conn_type: "local".to_string(),
            url: url.to_string(),
            token: token.to_string(),
            name: "Local".to_string(),
            managed: true,
        }],
        active_connection_id: "local".to_string(),
    }
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
fn read_connections() -> Result<String, String> {
    let dir = ensure_config_dir()?;
    let path = dir.join("connections.json");
    if path.exists() {
        std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read connections.json: {}", e))
    } else {
        let defaults = default_connections("http://127.0.0.1:7890", "");
        serde_json::to_string(&defaults)
            .map_err(|e| format!("failed to serialize default connections: {}", e))
    }
}

#[tauri::command]
fn write_connections(json: String) -> Result<(), String> {
    let dir = ensure_config_dir()?;
    let path = dir.join("connections.json");
    std::fs::write(&path, json)
        .map_err(|e| format!("failed to write connections.json: {}", e))
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

// ── Sidecar state ─────────────────────────────────────────────────────────────

pub struct SidecarState {
    pub child: Option<tauri_plugin_shell::process::CommandChild>,
    pub port: u16,
    pub token: String,
}

impl Default for SidecarState {
    fn default() -> Self {
        Self {
            child: None,
            port: 0,
            token: String::new(),
        }
    }
}

// ── Port / health helpers ─────────────────────────────────────────────────────

fn find_available_port(start: u16, end: u16) -> Option<u16> {
    for port in start..=end {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

async fn wait_for_health(port: u16, max_retries: u32) -> Result<(), String> {
    for attempt in 0..max_retries {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        match tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
            Ok(_) => {
                eprintln!("[sidecar] daemon is ready on port {} (attempt {})", port, attempt + 1);
                return Ok(());
            }
            Err(_) => {
                eprintln!("[sidecar] waiting for daemon on port {} (attempt {})", port, attempt + 1);
            }
        }
    }
    Err(format!("daemon did not become ready on port {} after {} retries", port, max_retries))
}

// ── Sidecar lifecycle ─────────────────────────────────────────────────────────

async fn start_sidecar(handle: tauri::AppHandle) {
    // 1. Generate random token
    let mut token_bytes = [0u8; 32];
    use rand::RngCore;
    rand::rng().fill_bytes(&mut token_bytes);
    let token = base64::engine::general_purpose::STANDARD.encode(&token_bytes);

    // 2. Find available port
    let port = match find_available_port(7890, 7899) {
        Some(p) => p,
        None => {
            eprintln!("[sidecar] no available port in range 7890-7899");
            return;
        }
    };

    eprintln!("[sidecar] starting claude-daemon on port {}", port);

    // 3. Spawn sidecar
    let shell = handle.shell();
    let sidecar_result = shell
        .sidecar("claude-daemon")
        .map_err(|e| format!("sidecar command error: {}", e))
        .and_then(|cmd| {
            cmd.args([
                "--port",
                &port.to_string(),
                "--token",
                &token,
                "--bind",
                "127.0.0.1",
            ])
            .spawn()
            .map_err(|e| format!("failed to spawn sidecar: {}", e))
        });

    let (mut rx, child) = match sidecar_result {
        Ok((rx, child)) => (rx, child),
        Err(e) => {
            eprintln!("[sidecar] {}", e);
            return;
        }
    };

    // 4. Store child handle
    {
        let state = handle.state::<Mutex<SidecarState>>();
        let mut s = state.lock().unwrap();
        s.child = Some(child);
        s.port = port;
        s.token = token.clone();
    }

    // 5. Wait for readiness
    if let Err(e) = wait_for_health(port, 10).await {
        eprintln!("[sidecar] {}", e);
    }

    // 6. Update connections.json
    let url = format!("http://127.0.0.1:{}", port);
    match (|| -> Result<(), String> {
        let dir = ensure_config_dir()?;
        let path = dir.join("connections.json");
        let mut file: ConnectionsFile = if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .map_err(|e| format!("read connections.json: {}", e))?;
            serde_json::from_str(&raw)
                .map_err(|e| format!("parse connections.json: {}", e))?
        } else {
            default_connections(&url, &token)
        };

        // Update or insert the local entry
        if let Some(entry) = file.connections.iter_mut().find(|c| c.conn_type == "local") {
            entry.url = url.clone();
            entry.token = token.clone();
        } else {
            file.connections.push(ConnectionEntry {
                id: "local".to_string(),
                conn_type: "local".to_string(),
                url: url.clone(),
                token: token.clone(),
                name: "Local".to_string(),
            managed: true,
            });
        }

        let serialized = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("serialize connections: {}", e))?;
        std::fs::write(&path, serialized)
            .map_err(|e| format!("write connections.json: {}", e))?;
        Ok(())
    })() {
        Ok(()) => eprintln!("[sidecar] connections.json updated"),
        Err(e) => eprintln!("[sidecar] failed to update connections.json: {}", e),
    }

    // 7. Drain stdout/stderr to prevent pipe buffer deadlock
    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    eprintln!("[claude-daemon stdout] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Stderr(line) => {
                    eprintln!("[claude-daemon stderr] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Error(e) => {
                    eprintln!("[claude-daemon error] {}", e);
                }
                CommandEvent::Terminated(status) => {
                    eprintln!("[claude-daemon] terminated: {:?}", status);
                    break;
                }
                _ => {}
            }
        }
    });
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(SidecarState::default()))
        .invoke_handler(tauri::generate_handler![
            get_config_dir,
            read_connections,
            write_connections,
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
        ])
        .setup(|app| {
            let claude_home = dirs_next::home_dir()
                .ok_or_else(|| "cannot determine home directory".to_string())?
                .join(".claude");
            let app_state = crate::state::AppState::new(claude_home);
            app.manage(app_state);

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                start_sidecar(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
