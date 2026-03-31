use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
};
use claude_types::{
    ErrorResponse,
    plugins::{
        AvailablePlugin, BlocklistFile, CommandRequest, InstalledPluginsFile, KnownMarketplace,
        MarketplaceInfo, MarketplaceManifest, PluginInfo,
    },
};
use claude_config::write::write_settings;
use serde::Deserialize;

use crate::{
    executor::{execute_claude_command, new_request_id},
    state::AppState,
};

// ---------------------------------------------------------------------------
// Request body types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ToggleRequest {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct InstallRequest {
    pub name: String,
    pub marketplace: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMarketplaceRequest {
    pub repo: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read and parse a JSON file, returning a default value if the file is missing
/// or unparseable.
fn read_json_file_or_default<T: Default + serde::de::DeserializeOwned>(path: &std::path::Path) -> T {
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
#[derive(Debug, Deserialize)]
struct PluginManifest {
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
// GET /api/v1/plugins
// ---------------------------------------------------------------------------

pub async fn list_plugins(
    Extension(state): Extension<AppState>,
) -> Json<Vec<PluginInfo>> {
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

    // installed.plugins maps marketplace_id -> Vec<InstalledPlugin>.
    // Each InstalledPlugin has a `scope` field which is the plugin name.
    // The canonical id is `{scope}@{marketplace_id}`.
    for (marketplace_id, plugins) in &installed.plugins {
        for plugin in plugins {
            let id = format!("{}@{}", plugin.scope, marketplace_id);
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

    Json(result)
}

// ---------------------------------------------------------------------------
// POST /api/v1/plugins/:id/toggle
// ---------------------------------------------------------------------------

pub async fn toggle_plugin(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ToggleRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Read → modify → write settings atomically.
    let settings_path = state.inner.claude_home.join("settings.json");

    let mut settings = state.inner.user_settings.read().await.clone();

    let enabled_plugins = settings.enabled_plugins.get_or_insert_with(HashMap::new);
    enabled_plugins.insert(id, body.enabled);

    // Write to disk.
    write_settings(&settings_path, &settings).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "WRITE_ERROR".to_string(),
                message: format!("Failed to write settings: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    // Update in-memory cache.
    *state.inner.user_settings.write().await = settings;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// GET /api/v1/marketplaces
// ---------------------------------------------------------------------------

pub async fn list_marketplaces(
    Extension(state): Extension<AppState>,
) -> Json<Vec<MarketplaceInfo>> {
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

    Json(result)
}

// ---------------------------------------------------------------------------
// GET /api/v1/marketplaces/:id/plugins
// ---------------------------------------------------------------------------

pub async fn browse_marketplace_plugins(
    Extension(state): Extension<AppState>,
    Path(marketplace_id): Path<String>,
) -> Result<Json<Vec<AvailablePlugin>>, (StatusCode, Json<ErrorResponse>)> {
    let plugins_dir = state.inner.claude_home.join("plugins");

    let known: HashMap<String, KnownMarketplace> =
        read_json_file_or_default(&plugins_dir.join("known_marketplaces.json"));

    let marketplace = known.get(&marketplace_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "MARKETPLACE_NOT_FOUND".to_string(),
                message: format!("Marketplace '{}' not found", marketplace_id),
                validation_errors: vec![],
            }),
        )
    })?;

    let install_location = marketplace.install_location.as_deref().ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "MARKETPLACE_NOT_INSTALLED".to_string(),
                message: format!("Marketplace '{}' has no install location", marketplace_id),
                validation_errors: vec![],
            }),
        )
    })?;

    let manifest_path = PathBuf::from(install_location)
        .join(".claude-plugin")
        .join("marketplace.json");

    let manifest: MarketplaceManifest =
        read_json_file_opt(&manifest_path).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "MARKETPLACE_MANIFEST_NOT_FOUND".to_string(),
                    message: format!(
                        "Could not read marketplace manifest for '{}'",
                        marketplace_id
                    ),
                    validation_errors: vec![],
                }),
            )
        })?;

    // Build a set of installed plugin scopes for this marketplace.
    let installed: InstalledPluginsFile =
        read_json_file_opt(&plugins_dir.join("installed_plugins.json"))
            .unwrap_or_else(|| InstalledPluginsFile {
                version: 1,
                plugins: HashMap::new(),
            });
    let installed_scopes: HashSet<String> = installed
        .plugins
        .get(&marketplace_id)
        .map(|plugins| plugins.iter().map(|p| p.scope.clone()).collect())
        .unwrap_or_default();

    let available: Vec<AvailablePlugin> = manifest
        .plugins
        .into_iter()
        .map(|p| {
            let installed_flag = installed_scopes.contains(&p.name);
            AvailablePlugin {
                name: p.name,
                marketplace: marketplace_id.clone(),
                installed: installed_flag,
                description: p.description,
                version: p.version,
                category: p.category,
            }
        })
        .collect();

    Ok(Json(available))
}

// ---------------------------------------------------------------------------
// POST /api/v1/plugins/install
// ---------------------------------------------------------------------------

pub async fn install_plugin(
    Extension(state): Extension<AppState>,
    Json(body): Json<InstallRequest>,
) -> Json<CommandRequest> {
    let request_id = new_request_id();
    let rid = request_id.clone();

    tokio::spawn(async move {
        let args = vec![
            "plugin",
            "install",
            body.name.as_str(),
            "--marketplace",
            body.marketplace.as_str(),
        ];
        let _ = execute_claude_command(&state, &args, &rid).await;
    });

    Json(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// POST /api/v1/plugins/:id/uninstall
// ---------------------------------------------------------------------------

pub async fn uninstall_plugin(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Json<CommandRequest> {
    let request_id = new_request_id();
    let rid = request_id.clone();

    tokio::spawn(async move {
        let args = vec!["plugin", "uninstall", id.as_str()];
        let _ = execute_claude_command(&state, &args, &rid).await;
    });

    Json(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// POST /api/v1/marketplaces
// ---------------------------------------------------------------------------

pub async fn add_marketplace(
    Extension(state): Extension<AppState>,
    Json(body): Json<AddMarketplaceRequest>,
) -> Json<CommandRequest> {
    let request_id = new_request_id();
    let rid = request_id.clone();

    tokio::spawn(async move {
        let args = vec![
            "plugin",
            "marketplace",
            "add",
            "--source",
            "github",
            "--repo",
            body.repo.as_str(),
        ];
        let _ = execute_claude_command(&state, &args, &rid).await;
    });

    Json(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/marketplaces/:id
// ---------------------------------------------------------------------------

pub async fn remove_marketplace(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Json<CommandRequest> {
    let request_id = new_request_id();
    let rid = request_id.clone();

    tokio::spawn(async move {
        let args = vec!["plugin", "marketplace", "remove", id.as_str()];
        let _ = execute_claude_command(&state, &args, &rid).await;
    });

    Json(CommandRequest { request_id })
}
