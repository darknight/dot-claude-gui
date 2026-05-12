use claude_config::{
    merge::{ConfigLayer, merge_layers},
    validate::validate_settings,
    write::write_settings,
};
use claude_types::{ConfigResponse, UpdateConfigRequest, settings::ConfigSource};
use tauri::State;

use crate::state::AppState;

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
    let settings_path = state.current_dir().await.join("settings.json");
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
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
        let on_disk = claude_config::parse::read_settings(&settings_path).unwrap();
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
}
