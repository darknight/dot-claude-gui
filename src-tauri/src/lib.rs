#[tauri::command]
fn get_daemon_url() -> String {
    "http://127.0.0.1:7890".to_string()
}

#[tauri::command]
fn get_daemon_token() -> Result<String, String> {
    let home = dirs_next::home_dir().ok_or("cannot find home directory")?;
    let token_path = home.join(".claude").join("daemon-token");
    std::fs::read_to_string(&token_path)
        .map_err(|e| format!("failed to read daemon token: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_daemon_url, get_daemon_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
