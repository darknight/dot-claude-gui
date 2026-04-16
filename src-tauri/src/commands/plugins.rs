use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use tauri::{AppHandle, State};

use claude_config::write::write_settings;
use claude_types::plugins::{
    AvailablePlugin, BlocklistFile, CommandRequest, InstalledPluginsFile,
    KnownMarketplace, MarketplaceInfo, MarketplaceManifest, PluginInfo,
};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers (copied verbatim from crates/claude-daemon/src/api/plugins.rs)
// ---------------------------------------------------------------------------

/// Read and parse a JSON file, returning a default value if the file is missing
/// or unparseable.
fn read_json_file_or_default<T: Default + serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> T {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => T::default(),
    }
}

/// Read and parse a JSON file, returning `None` if the file is missing.
fn read_json_file_opt<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Option<T> {
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Parse a plugin id like `"superpowers@claude-plugins-official"` into
/// `(name, marketplace)`.
fn split_plugin_id(id: &str) -> (String, String) {
    if let Some(pos) = id.find('@') {
        (id[..pos].to_string(), id[pos + 1..].to_string())
    } else {
        (id.to_string(), String::new())
    }
}

/// Try to read the description from `<install_path>/.claude-plugin/plugin.json`.
#[derive(Debug, serde::Deserialize)]
struct PluginManifest {
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

fn read_plugin_description(install_path: &str) -> Option<String> {
    let manifest_path = PathBuf::from(install_path)
        .join(".claude-plugin")
        .join("plugin.json");
    let contents = std::fs::read_to_string(manifest_path).ok()?;
    let manifest: PluginManifest = serde_json::from_str(&contents).ok()?;
    manifest.description
}

// ---------------------------------------------------------------------------
// list_plugins — ASYNC (reads user_settings RwLock for enabledPlugins)
// ---------------------------------------------------------------------------

pub(crate) async fn list_plugins_logic(state: &AppState) -> Vec<PluginInfo> {
    let plugins_dir = state.inner.claude_home.join("plugins");

    // Read installed_plugins.json (empty default if missing).
    let installed: InstalledPluginsFile =
        read_json_file_opt(&plugins_dir.join("installed_plugins.json"))
            .unwrap_or_else(|| InstalledPluginsFile {
                version: 1,
                plugins: HashMap::new(),
            });

    // Read blocklist.json (None if missing).
    let blocklist_opt: Option<BlocklistFile> =
        read_json_file_opt(&plugins_dir.join("blocklist.json"));
    let blocked_ids: HashSet<String> = blocklist_opt
        .map(|b| b.plugins.into_iter().map(|p| p.plugin).collect())
        .unwrap_or_default();

    // Read enabledPlugins from cached user settings.
    let enabled_map: HashMap<String, bool> = {
        let settings = state.inner.user_settings.read().await;
        settings.enabled_plugins.clone().unwrap_or_default()
    };

    let mut result = Vec::new();

    // installed.plugins maps plugin_key -> Vec<InstalledPlugin>.
    // Keys are "{plugin-name}@{marketplace-id}" (e.g. "superpowers@claude-plugins-official").
    // plugin.scope is the install scope ("user" | "project"), not the plugin name.
    for (plugin_key, plugins) in &installed.plugins {
        for plugin in plugins {
            let id = plugin_key.clone();
            let (name, marketplace) = split_plugin_id(&id);
            let enabled = enabled_map.get(&id).copied().unwrap_or(true);
            let blocked = blocked_ids.contains(&id);
            let description = read_plugin_description(&plugin.install_path);

            result.push(PluginInfo {
                id,
                name,
                marketplace,
                version: plugin.version.clone(),
                enabled,
                blocked,
                installed_at: plugin.installed_at.clone(),
                description,
            });
        }
    }

    result
}

#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    Ok(list_plugins_logic(&state).await)
}

// ---------------------------------------------------------------------------
// list_marketplaces — SYNC (only reads filesystem, no RwLock)
// ---------------------------------------------------------------------------

pub(crate) fn list_marketplaces_logic(state: &AppState) -> Vec<MarketplaceInfo> {
    let plugins_dir = state.inner.claude_home.join("plugins");

    let known: HashMap<String, KnownMarketplace> =
        read_json_file_or_default(&plugins_dir.join("known_marketplaces.json"));

    let mut result = Vec::new();

    for (id, marketplace) in &known {
        let (plugin_count, description) =
            if let Some(ref install_location) = marketplace.install_location {
                let manifest_path = PathBuf::from(install_location)
                    .join(".claude-plugin")
                    .join("marketplace.json");
                if let Some(manifest) = read_json_file_opt::<MarketplaceManifest>(&manifest_path) {
                    (manifest.plugins.len(), manifest.description)
                } else {
                    (0, None)
                }
            } else {
                (0, None)
            };

        result.push(MarketplaceInfo {
            id: id.clone(),
            repo: marketplace.source.repo.clone(),
            plugin_count,
            description,
            last_updated: marketplace.last_updated.clone(),
        });
    }

    result
}

#[tauri::command]
pub fn list_marketplaces(state: State<'_, AppState>) -> Result<Vec<MarketplaceInfo>, String> {
    Ok(list_marketplaces_logic(&state))
}

// ---------------------------------------------------------------------------
// get_marketplace_plugins — SYNC (only reads filesystem, no RwLock)
// ---------------------------------------------------------------------------

pub(crate) fn get_marketplace_plugins_logic(
    state: &AppState,
    marketplace_id: String,
) -> Result<Vec<AvailablePlugin>, String> {
    let plugins_dir = state.inner.claude_home.join("plugins");

    let known: HashMap<String, KnownMarketplace> =
        read_json_file_or_default(&plugins_dir.join("known_marketplaces.json"));

    let marketplace = known.get(&marketplace_id).ok_or_else(|| {
        format!(
            "marketplace_not_found: Marketplace '{}' not found",
            marketplace_id
        )
    })?;

    let install_location = marketplace.install_location.as_deref().ok_or_else(|| {
        format!(
            "marketplace_not_installed: Marketplace '{}' has no install location",
            marketplace_id
        )
    })?;

    let manifest_path = PathBuf::from(install_location)
        .join(".claude-plugin")
        .join("marketplace.json");

    let manifest: MarketplaceManifest = read_json_file_opt(&manifest_path).ok_or_else(|| {
        format!(
            "marketplace_manifest_not_found: Could not read marketplace manifest for '{}'",
            marketplace_id
        )
    })?;

    // Build name -> installed_version map for plugins from this marketplace.
    // Keys in installed_plugins.json are "name@marketplace" (e.g. "superpowers@claude-plugins-official").
    let installed: InstalledPluginsFile =
        read_json_file_opt(&plugins_dir.join("installed_plugins.json"))
            .unwrap_or_else(|| InstalledPluginsFile {
                version: 1,
                plugins: HashMap::new(),
            });
    let suffix = format!("@{}", marketplace_id);
    let installed_versions: HashMap<String, String> = installed
        .plugins
        .iter()
        .filter(|(key, _)| key.ends_with(&suffix))
        .flat_map(|(key, plugins)| {
            let name = key.split('@').next().unwrap_or(key).to_string();
            plugins.iter().map(move |p| (name.clone(), p.version.clone()))
        })
        .collect();

    let available: Vec<AvailablePlugin> = manifest
        .plugins
        .into_iter()
        .map(|p| {
            let installed_ver = installed_versions.get(&p.name);
            AvailablePlugin {
                name: p.name,
                marketplace: marketplace_id.clone(),
                installed: installed_ver.is_some(),
                installed_version: installed_ver.cloned(),
                description: p.description,
                version: p.version,
                category: p.category,
            }
        })
        .collect();

    Ok(available)
}

#[tauri::command]
pub fn get_marketplace_plugins(
    state: State<'_, AppState>,
    marketplace_id: String,
) -> Result<Vec<AvailablePlugin>, String> {
    get_marketplace_plugins_logic(&state, marketplace_id)
}

// ---------------------------------------------------------------------------
// toggle_plugin — ASYNC (reads/writes user_settings RwLock)
// ---------------------------------------------------------------------------

pub(crate) async fn toggle_plugin_logic(
    state: &AppState,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    // Read current settings.
    let mut settings = state.inner.user_settings.read().await.clone();

    // Modify enabledPlugins.
    let enabled_plugins = settings.enabled_plugins.get_or_insert_with(HashMap::new);
    enabled_plugins.insert(id, enabled);

    // Write to disk atomically.
    let settings_path = state.inner.claude_home.join("settings.json");
    write_settings(&settings_path, &settings)
        .map_err(|e| format!("write_error: Failed to write settings: {}", e))?;

    // Update in-memory cache.
    *state.inner.user_settings.write().await = settings;

    Ok(())
}

#[tauri::command]
pub async fn toggle_plugin(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    toggle_plugin_logic(&state, id, enabled).await
}

// ---------------------------------------------------------------------------
// install_plugin — streams `claude plugin install` via executor
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn install_plugin(
    app: AppHandle,
    _state: State<'_, AppState>,
    name: String,
    marketplace: String,
) -> Result<CommandRequest, String> {
    // claude plugin install <name>@<marketplace>
    let plugin_spec = format!("{}@{}", name, marketplace);
    let args = vec![
        "plugin".to_string(),
        "install".to_string(),
        plugin_spec,
    ];
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// uninstall_plugin — streams `claude plugin uninstall` via executor
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn uninstall_plugin(
    app: AppHandle,
    _state: State<'_, AppState>,
    id: String,
) -> Result<CommandRequest, String> {
    // Mirror daemon: claude plugin uninstall <id>
    let args = vec!["plugin".to_string(), "uninstall".to_string(), id];
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// add_marketplace — streams `claude plugin marketplace add` via executor
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn add_marketplace(
    app: AppHandle,
    _state: State<'_, AppState>,
    repo: String,
) -> Result<CommandRequest, String> {
    // Mirror daemon: claude plugin marketplace add --source github --repo <repo>
    let args = vec![
        "plugin".to_string(),
        "marketplace".to_string(),
        "add".to_string(),
        "--source".to_string(),
        "github".to_string(),
        "--repo".to_string(),
        repo,
    ];
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// remove_marketplace — streams `claude plugin marketplace remove` via executor
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn remove_marketplace(
    app: AppHandle,
    _state: State<'_, AppState>,
    id: String,
) -> Result<CommandRequest, String> {
    // Mirror daemon: claude plugin marketplace remove <id>
    let args = vec![
        "plugin".to_string(),
        "marketplace".to_string(),
        "remove".to_string(),
        id,
    ];
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use tempfile::tempdir;

    // 1. list_plugins_empty_when_no_installed_file
    #[tokio::test]
    async fn list_plugins_empty_when_no_installed_file() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let result = list_plugins_logic(&state).await;
        assert!(
            result.is_empty(),
            "expected empty list when installed_plugins.json is absent"
        );
    }

    // 2. list_plugins_reads_installed_plugins_json
    #[tokio::test]
    async fn list_plugins_reads_installed_plugins_json() {
        let dir = tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let installed_json = r#"{
            "version": 1,
            "plugins": {
                "claude-plugins-official": [
                    {
                        "scope": "superpowers",
                        "installPath": "/tmp/superpowers",
                        "version": "1.2.3",
                        "installedAt": "2024-01-01T00:00:00Z",
                        "lastUpdated": "2024-01-01T00:00:00Z"
                    }
                ]
            }
        }"#;
        std::fs::write(plugins_dir.join("installed_plugins.json"), installed_json).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_plugins_logic(&state).await;

        assert_eq!(result.len(), 1);
        let p = &result[0];
        assert_eq!(p.id, "superpowers@claude-plugins-official");
        assert_eq!(p.name, "superpowers");
        assert_eq!(p.marketplace, "claude-plugins-official");
        assert_eq!(p.version, "1.2.3");
        assert!(p.enabled, "plugin should be enabled by default");
        assert!(!p.blocked, "plugin should not be blocked");
        assert_eq!(p.installed_at, "2024-01-01T00:00:00Z");
    }

    // 3. list_plugins_respects_enabled_plugins_from_settings
    #[tokio::test]
    async fn list_plugins_respects_enabled_plugins_from_settings() {
        let dir = tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let installed_json = r#"{
            "version": 1,
            "plugins": {
                "my-marketplace": [
                    {
                        "scope": "my-plugin",
                        "installPath": "/tmp/my-plugin",
                        "version": "0.1.0",
                        "installedAt": "2024-06-01T00:00:00Z",
                        "lastUpdated": "2024-06-01T00:00:00Z"
                    }
                ]
            }
        }"#;
        std::fs::write(plugins_dir.join("installed_plugins.json"), installed_json).unwrap();

        // Write settings.json with enabledPlugins: { "my-plugin@my-marketplace": false }
        let settings_json = r#"{"enabledPlugins": {"my-plugin@my-marketplace": false}}"#;
        std::fs::write(dir.path().join("settings.json"), settings_json).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.load_user_settings().await.unwrap();

        let result = list_plugins_logic(&state).await;
        assert_eq!(result.len(), 1);
        assert!(!result[0].enabled, "plugin should be disabled per settings");
    }

    // 4. list_plugins_marks_blocked_plugins
    #[tokio::test]
    async fn list_plugins_marks_blocked_plugins() {
        let dir = tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let installed_json = r#"{
            "version": 1,
            "plugins": {
                "evil-marketplace": [
                    {
                        "scope": "bad-plugin",
                        "installPath": "/tmp/bad-plugin",
                        "version": "9.9.9",
                        "installedAt": "2024-01-01T00:00:00Z",
                        "lastUpdated": "2024-01-01T00:00:00Z"
                    }
                ]
            }
        }"#;
        std::fs::write(plugins_dir.join("installed_plugins.json"), installed_json).unwrap();

        let blocklist_json = r#"{
            "fetchedAt": "2024-01-01T00:00:00Z",
            "plugins": [
                {"plugin": "bad-plugin@evil-marketplace", "addedAt": "2024-01-01T00:00:00Z"}
            ]
        }"#;
        std::fs::write(plugins_dir.join("blocklist.json"), blocklist_json).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_plugins_logic(&state).await;

        assert_eq!(result.len(), 1);
        assert!(result[0].blocked, "plugin should be marked blocked");
    }

    // 5. list_marketplaces_empty_when_no_file
    #[test]
    fn list_marketplaces_empty_when_no_file() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let result = list_marketplaces_logic(&state);
        assert!(result.is_empty());
    }

    // 6. list_marketplaces_reads_known_marketplaces_json
    #[test]
    fn list_marketplaces_reads_known_marketplaces_json() {
        let dir = tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let json = r#"{
            "official": {
                "source": {"source": "github", "repo": "anthropics/claude-plugins-official"},
                "installLocation": null,
                "lastUpdated": "2024-01-01T00:00:00Z"
            }
        }"#;
        std::fs::write(plugins_dir.join("known_marketplaces.json"), json).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_marketplaces_logic(&state);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "official");
        assert_eq!(
            result[0].repo,
            "anthropics/claude-plugins-official"
        );
        assert_eq!(result[0].plugin_count, 0); // no install_location, so 0
    }

    // 7. get_marketplace_plugins_returns_error_when_marketplace_not_found
    #[test]
    fn get_marketplace_plugins_returns_error_when_marketplace_not_found() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let err =
            get_marketplace_plugins_logic(&state, "nonexistent".to_string()).unwrap_err();
        assert!(
            err.starts_with("marketplace_not_found:"),
            "expected 'marketplace_not_found:' error, got: {}",
            err
        );
    }

    // 8. toggle_plugin_persists_to_settings_and_updates_cache
    #[tokio::test]
    async fn toggle_plugin_persists_to_settings_and_updates_cache() {
        let dir = tempdir().unwrap();
        // Create a minimal valid settings.json
        std::fs::write(dir.path().join("settings.json"), "{}").unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.load_user_settings().await.unwrap();

        // Disable a plugin
        toggle_plugin_logic(&state, "my-plugin@marketplace".to_string(), false)
            .await
            .unwrap();

        // Check in-memory cache
        let settings = state.inner.user_settings.read().await;
        let enabled_map = settings.enabled_plugins.as_ref().unwrap();
        assert_eq!(
            enabled_map.get("my-plugin@marketplace").copied(),
            Some(false),
            "in-memory cache should reflect disabled state"
        );

        // Check on-disk
        drop(settings);
        let on_disk = std::fs::read_to_string(dir.path().join("settings.json")).unwrap();
        assert!(
            on_disk.contains("my-plugin@marketplace"),
            "settings.json should contain the plugin id"
        );
        assert!(
            on_disk.contains("false"),
            "settings.json should contain false"
        );
    }
}
