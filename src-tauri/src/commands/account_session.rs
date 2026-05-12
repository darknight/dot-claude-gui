// src-tauri/src/commands/account_session.rs
//
// IPCs that control "which account the GUI is currently inspecting".
// Distinct from CLAUDE_CONFIG_DIR injection (which is per-launch).

use std::path::PathBuf;
use serde::Serialize;
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountOverview {
    pub name: String,
    pub display_name: String,
    pub is_native: bool,
    pub config_dir: String,
    pub logged_in: bool,
    pub email: Option<String>,
    pub project_count: u32,
    pub plugin_count: u32,
    pub skill_count: u32,
}

/// Fetch a one-shot summary for the Account > Overview facet.
/// Does NOT switch the active account — read-only.
#[tauri::command]
pub async fn account_overview(name: String) -> Result<AccountOverview, String> {
    let cfg = read_config(&config_path()?)?;
    let acct = if name == DEFAULT_ACCOUNT_NAME {
        cfg.accounts.iter().find(|a| a.name == DEFAULT_ACCOUNT_NAME).cloned()
    } else {
        cfg.accounts.iter().find(|a| a.name == name).cloned()
    }
    .ok_or_else(|| format!("unknown_account: {name}"))?;

    let home = dirs_next::home_dir().ok_or("cannot determine home directory")?;
    let dir = account_dir(&home, &name);

    // Counts: directory listings, errors → 0.
    let project_count = std::fs::read_dir(dir.join("projects"))
        .map(|it| it.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count() as u32)
        .unwrap_or(0);

    let plugin_count = read_plugin_count(&dir);

    // Match `commands::skills::scan_skills_dir`: a real skill is a subdir
    // containing SKILL.md. Bare subdirs don't count.
    let skill_count = std::fs::read_dir(dir.join("skills"))
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
                .count() as u32
        })
        .unwrap_or(0);

    let (logged_in, email) = read_oauth_status(&dir);

    Ok(AccountOverview {
        name: acct.name,
        display_name: acct.display_name,
        is_native: acct.is_native,
        config_dir: dir.to_string_lossy().to_string(),
        logged_in,
        email,
        project_count,
        plugin_count,
        skill_count,
    })
}

fn read_plugin_count(dir: &std::path::Path) -> u32 {
    // installed_plugins.json shape: { "plugins": { <marketplace>: [InstalledPlugin, ...], ... } }
    // — see crates/claude-types/src/plugins.rs::InstalledPluginsFile.
    let path = dir.join("plugins").join("installed_plugins.json");
    let Ok(bytes) = std::fs::read(&path) else { return 0; };
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_slice(&bytes) else { return 0; };
    json.get("plugins")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.values()
                .filter_map(|v| v.as_array())
                .map(|a| a.len() as u32)
                .sum()
        })
        .unwrap_or(0)
}

fn read_oauth_status(dir: &std::path::Path) -> (bool, Option<String>) {
    // .claude.json sits in the account dir for non-default accounts;
    // for default, it's at ~/.claude.json (one level up from ~/.claude/).
    let claude_json = if dir.ends_with(".claude") {
        // default account: ~/.claude.json
        dir.parent().map(|p| p.join(".claude.json"))
    } else {
        Some(dir.join(".claude.json"))
    };
    let Some(path) = claude_json else { return (false, None); };
    let Ok(bytes) = std::fs::read(&path) else { return (false, None); };
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_slice(&bytes) else { return (false, None); };
    let oauth = json.get("oauthAccount");
    let logged_in = oauth.is_some();
    let email = oauth.and_then(|o| o.get("emailAddress")).and_then(|v| v.as_str()).map(String::from);
    (logged_in, email)
}
