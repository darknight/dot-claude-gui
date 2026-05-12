// src-tauri/src/commands/account_session.rs
//
// IPCs that control "which account the GUI is currently inspecting".
// Distinct from CLAUDE_CONFIG_DIR injection (which is per-launch).

use std::path::PathBuf;
use tauri::State;

use crate::app_config::{account_dir, read_config, DEFAULT_ACCOUNT_NAME};
use crate::state::AppState;

fn config_path() -> Result<PathBuf, String> {
    let dir = dirs_next::home_dir()
        .ok_or("cannot determine home directory")?
        .join(".dot-claude-gui");
    Ok(dir.join("config.json"))
}

/// Switch the active account. Validates `name` against `config.json.accounts`.
/// On success, the new dir is `~/.claude/` (for `default`) or
/// `~/.dot-claude-gui/accounts/<name>/`. Reloads the user-settings cache so
/// subsequent reads serve the new account.
#[tauri::command]
pub async fn set_active_account(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Validate against config.json
    let cfg = read_config(&config_path()?)?;
    let known = name == DEFAULT_ACCOUNT_NAME
        || cfg.accounts.iter().any(|a| a.name == name);
    if !known {
        return Err(format!("unknown_account: {name}"));
    }

    let home = dirs_next::home_dir().ok_or("cannot determine home directory")?;
    let new_dir = account_dir(&home, &name);

    state.set_active_account_dir(new_dir.clone()).await;

    // Refresh the user-settings cache for the new dir. Errors are non-fatal
    // (account may not yet have a settings.json on first visit).
    if let Err(e) = state.load_user_settings().await {
        tracing::warn!("failed to reload user settings after account switch: {e}");
    }

    Ok(new_dir.to_string_lossy().to_string())
}
