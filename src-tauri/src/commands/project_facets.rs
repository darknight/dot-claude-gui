// src-tauri/src/commands/project_facets.rs
//
// Path-keyed IPCs for reading/writing a project's .claude/settings.json
// directly (NOT via active-account state). Used by Projects mode facets.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response / request types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSettingsResponse {
    pub path: String,
    pub exists: bool,
    pub settings: claude_types::Settings,
}

#[derive(Debug, Deserialize)]
pub struct WriteProjectSettingsRequest {
    pub project_path: String,
    pub settings: claude_types::Settings,
}

// ---------------------------------------------------------------------------
// Pure helpers (pub(crate) for tests and potential internal reuse)
// ---------------------------------------------------------------------------

/// Returns `<project_path>/.claude/settings.json`.
pub(crate) fn project_settings_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".claude").join("settings.json")
}

/// Read the settings for a given project path.
///
/// - If the file does not exist, returns `{exists: false, settings: default, …}`.
/// - If the file exists, parses JSON and returns `{exists: true, settings: …}`.
pub(crate) fn read_settings_for_path(project_path: &str) -> Result<ProjectSettingsResponse, String> {
    let path = project_settings_path(project_path);
    if !path.exists() {
        return Ok(ProjectSettingsResponse {
            path: path.to_string_lossy().to_string(),
            exists: false,
            settings: claude_types::Settings::default(),
        });
    }

    let bytes = std::fs::read(&path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;
    let settings: claude_types::Settings = serde_json::from_slice(&bytes)
        .map_err(|e| format!("parse {}: {e}", path.display()))?;

    Ok(ProjectSettingsResponse {
        path: path.to_string_lossy().to_string(),
        exists: true,
        settings,
    })
}

/// Write settings atomically to `<project_path>/.claude/settings.json`.
///
/// Creates `.claude/` if it does not exist. Uses tempfile → rename for atomicity.
pub(crate) fn write_settings_for_path(
    project_path: &str,
    settings: &claude_types::Settings,
) -> Result<(), String> {
    use std::io::Write as _;

    let path = project_settings_path(project_path);
    let dir = path.parent()
        .ok_or_else(|| "settings path has no parent directory".to_string())?;

    std::fs::create_dir_all(dir)
        .map_err(|e| format!("mkdir {}: {e}", dir.display()))?;

    let json = serde_json::to_vec_pretty(settings)
        .map_err(|e| format!("serialize settings: {e}"))?;

    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| format!("create tempfile in {}: {e}", dir.display()))?;
    tmp.write_all(&json)
        .map_err(|e| format!("write tempfile: {e}"))?;
    tmp.persist(&path)
        .map_err(|e| format!("rename tempfile to {}: {e}", path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri IPC commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn project_read_settings(
    _state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectSettingsResponse, String> {
    read_settings_for_path(&project_path)
}

#[tauri::command]
pub async fn project_write_settings(
    _state: State<'_, AppState>,
    request: WriteProjectSettingsRequest,
) -> Result<(), String> {
    write_settings_for_path(&request.project_path, &request.settings)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn project_settings_path_resolves_under_dot_claude() {
        let proj = tempdir().unwrap();
        let p = project_settings_path(proj.path().to_str().unwrap());
        assert_eq!(p, proj.path().join(".claude").join("settings.json"));
    }

    #[test]
    fn read_missing_settings_returns_default_with_exists_false() {
        let proj = tempdir().unwrap();
        let resp = read_settings_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(!resp.exists);
        assert_eq!(
            resp.path,
            proj.path().join(".claude").join("settings.json").to_string_lossy()
        );
    }

    #[test]
    fn write_then_read_round_trips() {
        let proj = tempdir().unwrap();
        let mut s = claude_types::Settings::default();
        s.env = Some(
            [("FOO".to_string(), "bar".to_string())]
                .into_iter()
                .collect(),
        );
        write_settings_for_path(proj.path().to_str().unwrap(), &s).unwrap();
        let resp = read_settings_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(resp.exists);
        assert_eq!(resp.settings.env.as_ref().unwrap().get("FOO").unwrap(), "bar");
    }

    #[test]
    fn write_creates_dot_claude_dir_if_missing() {
        let proj = tempdir().unwrap();
        assert!(!proj.path().join(".claude").exists());
        write_settings_for_path(
            proj.path().to_str().unwrap(),
            &claude_types::Settings::default(),
        )
        .unwrap();
        assert!(proj.path().join(".claude").join("settings.json").exists());
    }
}
