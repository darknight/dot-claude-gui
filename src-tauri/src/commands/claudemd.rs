use std::path::PathBuf;

use claude_types::claudemd::{ClaudeMdFile, ClaudeMdFileDetail};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Path resolution helper
// ---------------------------------------------------------------------------

async fn resolve_claudemd_path(state: &AppState, id: &str) -> Result<PathBuf, String> {
    if id == "global" {
        return Ok(state.inner.claude_home.join("CLAUDE.md"));
    }

    if let Some(project_id) = id.strip_prefix("project:") {
        let projects = state.inner.projects.read().await;
        let project = projects
            .iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| format!("not_found: Project '{}' not found", project_id))?;
        return Ok(project.path.join("CLAUDE.md"));
    }

    Err(format!("invalid_id: Invalid CLAUDE.md id format: '{}'", id))
}

// ---------------------------------------------------------------------------
// Logic helpers (testable without Tauri State)
// ---------------------------------------------------------------------------

pub(crate) async fn list_claudemd_files_logic(state: &AppState) -> Vec<ClaudeMdFile> {
    let mut result = Vec::new();

    let global_path = state.inner.claude_home.join("CLAUDE.md");
    result.push(ClaudeMdFile {
        id: "global".to_string(),
        scope: "global".to_string(),
        filename: "CLAUDE.md".to_string(),
        path: global_path.to_string_lossy().into_owned(),
        project_id: None,
        project_name: None,
        exists: global_path.exists(),
    });

    let projects = state.inner.projects.read().await;
    for project in projects.iter() {
        let project_path = project.path.join("CLAUDE.md");
        let id = format!("project:{}", project.id);
        result.push(ClaudeMdFile {
            id,
            scope: "project".to_string(),
            filename: "CLAUDE.md".to_string(),
            path: project_path.to_string_lossy().into_owned(),
            project_id: Some(project.id.clone()),
            project_name: Some(project.name.clone()),
            exists: project_path.exists(),
        });
    }

    result
}

pub(crate) async fn get_claudemd_file_logic(
    state: &AppState,
    id: String,
) -> Result<ClaudeMdFileDetail, String> {
    let file_path = resolve_claudemd_path(state, &id).await?;

    let content = std::fs::read_to_string(&file_path)
        .map_err(|_| format!("not_found: CLAUDE.md not found at '{}'", file_path.display()))?;

    let scope = if id == "global" { "global" } else { "project" };
    let project_id = id.strip_prefix("project:").map(|s| s.to_string());

    Ok(ClaudeMdFileDetail {
        id,
        scope: scope.to_string(),
        filename: "CLAUDE.md".to_string(),
        path: file_path.to_string_lossy().into_owned(),
        content,
        project_id,
    })
}

pub(crate) async fn update_claudemd_file_logic(
    state: &AppState,
    id: String,
    content: String,
) -> Result<(), String> {
    let file_path = resolve_claudemd_path(state, &id).await?;

    claude_config::write::atomic_write(&file_path, content.as_bytes())
        .map_err(|e| format!("write_error: Failed to write CLAUDE.md: {}", e))?;

    Ok(())
}

pub(crate) async fn delete_claudemd_file_logic(
    state: &AppState,
    id: String,
) -> Result<(), String> {
    let file_path = resolve_claudemd_path(state, &id).await?;

    std::fs::remove_file(&file_path)
        .map_err(|_| format!("not_found: CLAUDE.md not found at '{}'", file_path.display()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri command shims
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_claudemd_files(
    state: State<'_, AppState>,
) -> Result<Vec<ClaudeMdFile>, String> {
    Ok(list_claudemd_files_logic(&state).await)
}

#[tauri::command]
pub async fn get_claudemd_file(
    state: State<'_, AppState>,
    id: String,
) -> Result<ClaudeMdFileDetail, String> {
    get_claudemd_file_logic(&state, id).await
}

#[tauri::command]
pub async fn update_claudemd_file(
    state: State<'_, AppState>,
    id: String,
    content: String,
) -> Result<(), String> {
    update_claudemd_file_logic(&state, id, content).await
}

#[tauri::command]
pub async fn delete_claudemd_file(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    delete_claudemd_file_logic(&state, id).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, ProjectInfo};
    use tempfile::tempdir;

    // 1. list_returns_global_entry_even_when_missing
    #[tokio::test]
    async fn list_returns_global_entry_even_when_missing() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let result = list_claudemd_files_logic(&state).await;

        // Global entry is always present
        assert_eq!(result.len(), 1, "expected exactly 1 entry (global)");
        let global = &result[0];
        assert_eq!(global.id, "global");
        assert_eq!(global.scope, "global");
        assert!(!global.exists, "global CLAUDE.md should not exist yet");
    }

    // 2. get_global_returns_content_when_exists
    #[tokio::test]
    async fn get_global_returns_content_when_exists() {
        let dir = tempdir().unwrap();
        let global_path = dir.path().join("CLAUDE.md");
        std::fs::write(&global_path, "# My global instructions\n").unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let detail = get_claudemd_file_logic(&state, "global".to_string())
            .await
            .unwrap();

        assert_eq!(detail.id, "global");
        assert_eq!(detail.scope, "global");
        assert_eq!(detail.content, "# My global instructions\n");
        assert!(detail.project_id.is_none());
    }

    // 3. update_global_writes_file
    #[tokio::test]
    async fn update_global_writes_file() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        update_claudemd_file_logic(
            &state,
            "global".to_string(),
            "# Written by test\n".to_string(),
        )
        .await
        .unwrap();

        let on_disk = std::fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
        assert_eq!(on_disk, "# Written by test\n");
    }

    // 4. delete_global_removes_file
    #[tokio::test]
    async fn delete_global_removes_file() {
        let dir = tempdir().unwrap();
        let global_path = dir.path().join("CLAUDE.md");
        std::fs::write(&global_path, "# To be deleted\n").unwrap();
        assert!(global_path.exists());

        let state = AppState::new(dir.path().to_path_buf());
        delete_claudemd_file_logic(&state, "global".to_string())
            .await
            .unwrap();

        assert!(!global_path.exists(), "CLAUDE.md should be deleted");
    }

    // 5. get_returns_not_found_when_file_missing
    #[tokio::test]
    async fn get_returns_not_found_when_file_missing() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let err = get_claudemd_file_logic(&state, "global".to_string())
            .await
            .unwrap_err();

        assert!(
            err.starts_with("not_found:"),
            "expected 'not_found:' error, got: {}",
            err
        );
    }

    // 6. project_entry_appears_in_list
    #[tokio::test]
    async fn project_entry_appears_in_list() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().join("my-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        let state = AppState::new(dir.path().to_path_buf());

        // Register a project manually
        {
            let mut projects = state.inner.projects.write().await;
            projects.push(ProjectInfo {
                id: "proj-001".to_string(),
                path: project_dir.clone(),
                name: "my-project".to_string(),
            });
        }

        let result = list_claudemd_files_logic(&state).await;
        assert_eq!(result.len(), 2, "expected global + 1 project entry");

        let proj = result.iter().find(|e| e.scope == "project").unwrap();
        assert_eq!(proj.id, "project:proj-001");
        assert_eq!(proj.project_id.as_deref(), Some("proj-001"));
        assert_eq!(proj.project_name.as_deref(), Some("my-project"));
        assert!(!proj.exists);
    }
}
