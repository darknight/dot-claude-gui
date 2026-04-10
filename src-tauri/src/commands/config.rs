use std::collections::HashMap;

use claude_config::{
    merge::{ConfigLayer, merge_layers},
    parse::read_settings,
    validate::validate_settings,
    write::write_settings,
};
use claude_types::{ConfigResponse, UpdateConfigRequest, settings::ConfigSource};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Additional response type (mirrors daemon's EffectiveConfigResponse)
// ---------------------------------------------------------------------------

/// Response for the effective config endpoint — includes per-field source tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveConfigResponse {
    pub settings: claude_types::Settings,
    pub field_sources: HashMap<String, ConfigSource>,
}

// ---------------------------------------------------------------------------
// get_user_config
// ---------------------------------------------------------------------------

pub(crate) async fn get_user_config_logic(state: &AppState) -> Result<ConfigResponse, String> {
    let settings = state.inner.user_settings.read().await.clone();
    Ok(ConfigResponse {
        settings,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn get_user_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    get_user_config_logic(&state).await
}

// ---------------------------------------------------------------------------
// update_user_config
// ---------------------------------------------------------------------------

pub(crate) async fn update_user_config_logic(
    state: &AppState,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    // Merge current settings with the incoming update (update wins).
    let current = state.inner.user_settings.read().await.clone();
    let merged = merge_layers(&[
        ConfigLayer { source: ConfigSource::User, settings: current },
        ConfigLayer { source: ConfigSource::User, settings: req.settings },
    ]);

    // Validate the merged result.
    let errors = validate_settings(&merged.settings);
    if !errors.is_empty() {
        let msgs: Vec<String> = errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        return Err(format!("validation: {}", msgs.join("; ")));
    }

    // Write atomically to disk.
    let settings_path = state.inner.claude_home.join("settings.json");
    write_settings(&settings_path, &merged.settings)
        .map_err(|e| format!("write: failed to write settings: {}", e))?;

    // Update the in-memory cache.
    *state.inner.user_settings.write().await = merged.settings.clone();

    Ok(ConfigResponse {
        settings: merged.settings,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn update_user_config(
    state: State<'_, AppState>,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    update_user_config_logic(&state, req).await
}

// ---------------------------------------------------------------------------
// get_project_config
// ---------------------------------------------------------------------------

pub(crate) async fn get_project_config_logic(
    state: &AppState,
    project_id: String,
) -> Result<ConfigResponse, String> {
    // Look up the project by ID.
    let project_path = {
        let projects = state.inner.projects.read().await;
        projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
    };

    let project_path = project_path
        .ok_or_else(|| format!("not_found: project '{}' not found", project_id))?;

    // Read settings from <project>/.claude/settings.json.
    let settings_path = project_path.join(".claude").join("settings.json");
    let settings = read_settings(&settings_path)
        .map_err(|e| format!("internal: failed to read project settings: {}", e))?;

    Ok(ConfigResponse {
        settings,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn get_project_config(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ConfigResponse, String> {
    get_project_config_logic(&state, project_id).await
}

// ---------------------------------------------------------------------------
// update_project_config
// ---------------------------------------------------------------------------

pub(crate) async fn update_project_config_logic(
    state: &AppState,
    project_id: String,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    // Look up the project by ID.
    let project_path = {
        let projects = state.inner.projects.read().await;
        projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.path.clone())
    };

    let project_path = project_path
        .ok_or_else(|| format!("not_found: project '{}' not found", project_id))?;

    // Read current project settings from <project>/.claude/settings.json.
    let settings_path = project_path.join(".claude").join("settings.json");
    let current = read_settings(&settings_path)
        .map_err(|e| format!("internal: failed to read project settings: {}", e))?;

    // Merge current settings with the incoming update (update wins).
    let merged = merge_layers(&[
        ConfigLayer { source: ConfigSource::Project, settings: current },
        ConfigLayer { source: ConfigSource::Project, settings: req.settings },
    ]);

    // Validate the merged result.
    let errors = validate_settings(&merged.settings);
    if !errors.is_empty() {
        let msgs: Vec<String> = errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        return Err(format!("validation: {}", msgs.join("; ")));
    }

    // Write atomically to disk (write_settings creates parent dirs via atomic_write).
    write_settings(&settings_path, &merged.settings)
        .map_err(|e| format!("write: failed to write project settings: {}", e))?;

    Ok(ConfigResponse {
        settings: merged.settings,
        last_modified: None,
        version: None,
    })
}

#[tauri::command]
pub async fn update_project_config(
    state: State<'_, AppState>,
    project_id: String,
    req: UpdateConfigRequest,
) -> Result<ConfigResponse, String> {
    update_project_config_logic(&state, project_id, req).await
}

// ---------------------------------------------------------------------------
// get_effective_config
// ---------------------------------------------------------------------------

pub(crate) async fn get_effective_config_logic(
    state: &AppState,
    project_id: String,
) -> Result<EffectiveConfigResponse, String> {
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
        let project_settings = read_settings(&project_settings_path)
            .map_err(|e| format!("internal: failed to read project settings: {}", e))?;
        layers.push(ConfigLayer {
            source: ConfigSource::Project,
            settings: project_settings,
        });

        // Layer 3: local settings from <project>/.claude/settings.local.json.
        let local_settings_path = path.join(".claude").join("settings.local.json");
        let local_settings = read_settings(&local_settings_path)
            .map_err(|e| format!("internal: failed to read local settings: {}", e))?;
        layers.push(ConfigLayer {
            source: ConfigSource::Local,
            settings: local_settings,
        });
    }

    let merged = merge_layers(&layers);

    Ok(EffectiveConfigResponse {
        settings: merged.settings,
        field_sources: merged.field_sources,
    })
}

#[tauri::command]
pub async fn get_effective_config(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<EffectiveConfigResponse, String> {
    get_effective_config_logic(&state, project_id).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ProjectInfo;
    use claude_types::Settings;
    use tempfile::tempdir;

    // 1. get_user_config_returns_cached_settings
    #[tokio::test]
    async fn get_user_config_returns_cached_settings() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        std::fs::write(
            &settings_path,
            r#"{"env": {"MYKEY": "myval"}}"#,
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        state.load_user_settings().await.unwrap();

        let result = get_user_config_logic(&state).await.unwrap();
        assert_eq!(
            result.settings.env.as_ref().and_then(|m| m.get("MYKEY")).map(String::as_str),
            Some("myval")
        );
    }

    // 2. update_user_config_writes_to_disk
    #[tokio::test]
    async fn update_user_config_writes_to_disk() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let mut env_map = std::collections::HashMap::new();
        env_map.insert("FOO".to_string(), "bar".to_string());
        let new_settings = Settings {
            env: Some(env_map),
            ..Default::default()
        };
        let req = UpdateConfigRequest {
            settings: new_settings,
            if_match: None,
        };

        let result = update_user_config_logic(&state, req).await.unwrap();

        // Verify returned settings
        assert_eq!(
            result.settings.env.as_ref().and_then(|m| m.get("FOO")).map(String::as_str),
            Some("bar")
        );

        // Verify the file was written to disk
        let settings_path = dir.path().join("settings.json");
        assert!(settings_path.exists(), "settings.json should exist on disk");
        let on_disk = read_settings(&settings_path).unwrap();
        assert_eq!(
            on_disk.env.as_ref().and_then(|m| m.get("FOO")).map(String::as_str),
            Some("bar")
        );

        // Verify cache was updated
        let cached = state.inner.user_settings.read().await;
        assert_eq!(
            cached.env.as_ref().and_then(|m| m.get("FOO")).map(String::as_str),
            Some("bar")
        );
    }

    // 3. get_project_config_returns_not_found_for_unknown_project
    #[tokio::test]
    async fn get_project_config_returns_not_found_for_unknown_project() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let err = get_project_config_logic(&state, "nonexistent-project".to_string())
            .await
            .unwrap_err();

        assert!(
            err.starts_with("not_found:"),
            "expected error starting with 'not_found:', got: {}",
            err
        );
    }

    // 4. get_project_config_reads_project_settings
    #[tokio::test]
    async fn get_project_config_reads_project_settings() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        // Create a project directory with settings
        let project_dir = dir.path().join("my-project");
        let claude_dir = project_dir.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        std::fs::write(
            claude_dir.join("settings.json"),
            r#"{"language": "fr"}"#,
        )
        .unwrap();

        // Register the project in state
        {
            let mut projects = state.inner.projects.write().await;
            projects.push(ProjectInfo {
                id: "proj-1".to_string(),
                path: project_dir,
                name: "My Project".to_string(),
            });
        }

        let result = get_project_config_logic(&state, "proj-1".to_string())
            .await
            .unwrap();
        assert_eq!(result.settings.language, Some("fr".to_string()));
    }
}
