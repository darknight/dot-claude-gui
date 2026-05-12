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
// ProjectClaudeMd response / request types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectClaudeMdResponse {
    pub path: String,
    pub exists: bool,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteProjectClaudeMdRequest {
    pub project_path: String,
    pub content: String,
}

// ---------------------------------------------------------------------------
// ProjectClaudeMd pure helpers
// ---------------------------------------------------------------------------

/// Returns `<project_path>/.claude/CLAUDE.md`.
pub(crate) fn project_claudemd_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join(".claude").join("CLAUDE.md")
}

/// Read the CLAUDE.md for a given project path.
///
/// - If the file does not exist, returns `{exists: false, content: "", …}`.
/// - If the file exists, returns `{exists: true, content: …}`.
pub(crate) fn read_claudemd_for_path(project_path: &str) -> Result<ProjectClaudeMdResponse, String> {
    let path = project_claudemd_path(project_path);
    if !path.exists() {
        return Ok(ProjectClaudeMdResponse {
            path: path.to_string_lossy().to_string(),
            exists: false,
            content: String::new(),
        });
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("read {}: {e}", path.display()))?;

    Ok(ProjectClaudeMdResponse {
        path: path.to_string_lossy().to_string(),
        exists: true,
        content,
    })
}

/// Write CLAUDE.md atomically to `<project_path>/.claude/CLAUDE.md`.
///
/// Creates `.claude/` if it does not exist. Uses tempfile → rename for atomicity.
pub(crate) fn write_claudemd_for_path(
    project_path: &str,
    content: &str,
) -> Result<(), String> {
    use std::io::Write as _;

    let path = project_claudemd_path(project_path);
    let dir = path.parent()
        .ok_or_else(|| "CLAUDE.md path has no parent directory".to_string())?;

    std::fs::create_dir_all(dir)
        .map_err(|e| format!("mkdir {}: {e}", dir.display()))?;

    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| format!("create tempfile in {}: {e}", dir.display()))?;
    tmp.write_all(content.as_bytes())
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

#[tauri::command]
pub async fn project_read_claudemd(
    _state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectClaudeMdResponse, String> {
    read_claudemd_for_path(&project_path)
}

#[tauri::command]
pub async fn project_write_claudemd(
    _state: State<'_, AppState>,
    request: WriteProjectClaudeMdRequest,
) -> Result<(), String> {
    write_claudemd_for_path(&request.project_path, &request.content)
}

// ---------------------------------------------------------------------------
// Project memory — account-scoped via binding
// ---------------------------------------------------------------------------

use claude_types::memory::MemoryFile;

/// Resolve `<account_dir>/projects/<encoded_path>/memory/` for a project path.
///
/// `encoded_path` = `project_path.replace('/', "-")` (Claude Code's convention).
pub(crate) async fn project_memory_dir(
    state: &AppState,
    project_path: &str,
) -> Result<PathBuf, String> {
    let account_dir = state.resolve_project_account_dir(project_path).await?;
    let encoded = project_path.replace('/', "-");
    Ok(account_dir.join("projects").join(encoded).join("memory"))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMemoryListResponse {
    pub path: String,
    pub files: Vec<MemoryFile>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectMemoryFileRequest {
    pub project_path: String,
    pub file_name: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteProjectMemoryRequest {
    pub project_path: String,
    pub file_name: String,
    pub content: String,
}

#[tauri::command]
pub async fn project_list_memory(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<ProjectMemoryListResponse, String> {
    let dir = project_memory_dir(&state, &project_path).await?;
    let files = if dir.exists() {
        crate::commands::memory::list_memory_files_in_dir(&dir)?
    } else {
        Vec::new()
    };
    Ok(ProjectMemoryListResponse {
        path: dir.to_string_lossy().into_owned(),
        files,
    })
}

#[tauri::command]
pub async fn project_read_memory_file(
    state: State<'_, AppState>,
    request: ProjectMemoryFileRequest,
) -> Result<String, String> {
    let dir = project_memory_dir(&state, &request.project_path).await?;
    let path = dir.join(&request.file_name);
    std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))
}

#[tauri::command]
pub async fn project_write_memory_file(
    state: State<'_, AppState>,
    request: WriteProjectMemoryRequest,
) -> Result<(), String> {
    let dir = project_memory_dir(&state, &request.project_path).await?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let path = dir.join(&request.file_name);
    claude_config::write::atomic_write(&path, request.content.as_bytes())
        .map_err(|e| format!("write {}: {e}", path.display()))
}

#[tauri::command]
pub async fn project_delete_memory_file(
    state: State<'_, AppState>,
    request: ProjectMemoryFileRequest,
) -> Result<(), String> {
    let dir = project_memory_dir(&state, &request.project_path).await?;
    let path = dir.join(&request.file_name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("remove {}: {e}", path.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Project plugins — account-scoped via binding
// ---------------------------------------------------------------------------

use claude_types::plugins::PluginInfo;

pub(crate) async fn list_plugins_for_project(
    state: &AppState,
    project_path: &str,
) -> Result<Vec<PluginInfo>, String> {
    let account_dir = state.resolve_project_account_dir(project_path).await?;
    let plugins_dir = account_dir.join("plugins");

    // Read bound account's user-layer settings.json directly to get its
    // enabled_plugins map. Empty map if missing or unparseable — matches the
    // behavior of the cached path in list_plugins_logic.
    let settings_path = account_dir.join("settings.json");
    let enabled_map = if settings_path.exists() {
        std::fs::read_to_string(&settings_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<claude_types::Settings>(&raw).ok())
            .and_then(|s| s.enabled_plugins)
            .unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    Ok(crate::commands::plugins::list_plugins_in_dir(&plugins_dir, &enabled_map))
}

#[tauri::command]
pub async fn project_list_plugins(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<Vec<PluginInfo>, String> {
    list_plugins_for_project(&state, &project_path).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Assert that the encoded-path rule (slashes → dashes) is applied correctly
    /// when building the memory dir. We test the pure path construction rule
    /// without needing to spin up a full AppState backed by $HOME.
    #[test]
    fn project_memory_dir_encodes_slashes_as_dashes() {
        let account_dir = std::path::PathBuf::from("/home/u/.dot-claude-gui/accounts/work");
        let encoded = "/Users/eric/code/foo".replace('/', "-");
        let expected = account_dir.join("projects").join(encoded).join("memory");
        assert_eq!(
            expected.to_string_lossy(),
            "/home/u/.dot-claude-gui/accounts/work/projects/-Users-eric-code-foo/memory"
        );
    }

    #[test]
    fn list_memory_files_in_dir_empty_when_missing() {
        let proj = tempdir().unwrap();
        // Memory dir does not exist:
        let result =
            crate::commands::memory::list_memory_files_in_dir(&proj.path().join("memory"));
        assert!(result.is_err(), "expected error for missing dir");
    }

    #[test]
    fn list_memory_files_in_dir_returns_md_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "x").unwrap();
        std::fs::write(dir.path().join("b.md"), "x").unwrap();
        std::fs::write(dir.path().join("c.txt"), "x").unwrap();
        let files = crate::commands::memory::list_memory_files_in_dir(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

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

    #[test]
    fn project_claudemd_path_resolves_under_dot_claude() {
        let proj = tempdir().unwrap();
        let p = project_claudemd_path(proj.path().to_str().unwrap());
        assert_eq!(p, proj.path().join(".claude").join("CLAUDE.md"));
    }

    #[test]
    fn read_missing_claudemd_returns_empty_with_exists_false() {
        let proj = tempdir().unwrap();
        let resp = read_claudemd_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(!resp.exists);
        assert_eq!(resp.content, "");
    }

    #[test]
    fn write_then_read_claudemd_round_trips() {
        let proj = tempdir().unwrap();
        write_claudemd_for_path(proj.path().to_str().unwrap(), "# Hello\n").unwrap();
        let resp = read_claudemd_for_path(proj.path().to_str().unwrap()).unwrap();
        assert!(resp.exists);
        assert_eq!(resp.content, "# Hello\n");
    }

    #[test]
    fn write_claudemd_creates_dot_claude_dir_if_missing() {
        let proj = tempdir().unwrap();
        assert!(!proj.path().join(".claude").exists());
        write_claudemd_for_path(proj.path().to_str().unwrap(), "x").unwrap();
        assert!(proj.path().join(".claude").join("CLAUDE.md").exists());
    }

    #[test]
    fn list_plugins_in_dir_returns_empty_when_no_installed_file() {
        let dir = tempdir().unwrap();
        // No installed_plugins.json:
        let result = crate::commands::plugins::list_plugins_in_dir(
            dir.path(),
            &std::collections::HashMap::new(),
        );
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn list_plugins_in_dir_reads_installed_file_and_applies_enabled_map() {
        let dir = tempdir().unwrap();
        // Match the on-disk format the existing module expects:
        // installed_plugins.json: { "version": 1, "plugins": { "<key>": [<entry>] } }
        let json = r#"{
            "version": 1,
            "plugins": {
                "my-plugin@my-marketplace": [{
                    "version": "1.0.0",
                    "installPath": "/tmp/nonexistent",
                    "installedAt": "2026-05-12T00:00:00Z",
                    "lastUpdated": "2026-05-12T00:00:00Z",
                    "scope": "user"
                }]
            }
        }"#;
        std::fs::write(dir.path().join("installed_plugins.json"), json).unwrap();

        // Enabled map sets my-plugin@my-marketplace to false:
        let mut enabled = std::collections::HashMap::new();
        enabled.insert("my-plugin@my-marketplace".to_string(), false);
        let result = crate::commands::plugins::list_plugins_in_dir(dir.path(), &enabled);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "my-plugin@my-marketplace");
        assert_eq!(result[0].enabled, false);
    }
}
