use std::collections::HashMap;

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
};
use claude_config::{
    merge::{ConfigLayer, merge_layers},
    parse::read_settings,
    validate::validate_settings,
    write::write_settings,
};
use claude_types::{
    ConfigResponse, ErrorResponse, UpdateConfigRequest,
    settings::ConfigSource,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Additional response types
// ---------------------------------------------------------------------------

/// Response for the effective config endpoint — includes per-field source tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveConfigResponse {
    pub settings: claude_types::Settings,
    pub field_sources: HashMap<String, ConfigSource>,
}

// ---------------------------------------------------------------------------
// GET /api/v1/config/user
// ---------------------------------------------------------------------------

/// Returns the cached user-level settings.
pub async fn get_user_config(
    Extension(state): Extension<AppState>,
) -> Json<ConfigResponse> {
    let settings = state.inner.user_settings.read().await.clone();
    Json(ConfigResponse {
        settings,
        last_modified: None,
        version: None,
    })
}

// ---------------------------------------------------------------------------
// PUT /api/v1/config/user
// ---------------------------------------------------------------------------

/// Updates the user config: merge → validate → atomic write → update cache.
pub async fn put_user_config(
    Extension(state): Extension<AppState>,
    Json(body): Json<UpdateConfigRequest>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Merge current settings with the incoming update (update wins).
    let current = state.inner.user_settings.read().await.clone();
    let merged = merge_layers(&[
        ConfigLayer { source: ConfigSource::User, settings: current },
        ConfigLayer { source: ConfigSource::User, settings: body.settings },
    ]);

    // Validate the merged result.
    let errors = validate_settings(&merged.settings);
    if !errors.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Settings validation failed".to_string(),
                validation_errors: errors,
            }),
        ));
    }

    // Write atomically to disk.
    let settings_path = state.inner.claude_home.join("settings.json");
    write_settings(&settings_path, &merged.settings).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "WRITE_ERROR".to_string(),
                message: format!("Failed to write settings: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    // Update the in-memory cache.
    *state.inner.user_settings.write().await = merged.settings.clone();

    Ok(Json(ConfigResponse {
        settings: merged.settings,
        last_modified: None,
        version: None,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/config/project/{project_id}
// ---------------------------------------------------------------------------

/// Returns the project-level settings read from disk.
pub async fn get_project_config(
    Extension(state): Extension<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Look up the project by ID.
    let project_path = {
        let projects = state.inner.projects.read().await;
        projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
    };

    let project_path = project_path.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "PROJECT_NOT_FOUND".to_string(),
                message: format!("Project '{}' not found", project_id),
                validation_errors: vec![],
            }),
        )
    })?;

    // Read settings from <project>/.claude/settings.json.
    let settings_path = project_path.join(".claude").join("settings.json");
    let settings = read_settings(&settings_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "READ_ERROR".to_string(),
                message: format!("Failed to read project settings: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(Json(ConfigResponse {
        settings,
        last_modified: None,
        version: None,
    }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/config/effective/{project_id}
// ---------------------------------------------------------------------------

/// Returns the merged effective config with per-field source tracking.
pub async fn get_effective_config(
    Extension(state): Extension<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<EffectiveConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Layer 1: user settings from cache.
    let user_settings = state.inner.user_settings.read().await.clone();
    let mut layers = vec![ConfigLayer {
        source: ConfigSource::User,
        settings: user_settings,
    }];

    // Resolve project path (optional — we don't error if the project is unknown).
    let project_path = {
        let projects = state.inner.projects.read().await;
        projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
    };

    if let Some(ref path) = project_path {
        // Layer 2: project settings from <project>/.claude/settings.json.
        let project_settings_path = path.join(".claude").join("settings.json");
        let project_settings = read_settings(&project_settings_path).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "READ_ERROR".to_string(),
                    message: format!("Failed to read project settings: {}", e),
                    validation_errors: vec![],
                }),
            )
        })?;
        layers.push(ConfigLayer {
            source: ConfigSource::Project,
            settings: project_settings,
        });

        // Layer 3: local settings from <project>/.claude/settings.local.json.
        let local_settings_path = path.join(".claude").join("settings.local.json");
        let local_settings = read_settings(&local_settings_path).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "READ_ERROR".to_string(),
                    message: format!("Failed to read local settings: {}", e),
                    validation_errors: vec![],
                }),
            )
        })?;
        layers.push(ConfigLayer {
            source: ConfigSource::Local,
            settings: local_settings,
        });
    }

    let merged = merge_layers(&layers);

    Ok(Json(EffectiveConfigResponse {
        settings: merged.settings,
        field_sources: merged.field_sources,
    }))
}
