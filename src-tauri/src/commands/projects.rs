use claude_types::{ProjectEntry, RegisterProjectRequest};
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// list_projects
// ---------------------------------------------------------------------------

pub(crate) async fn list_projects_logic(state: &AppState) -> Result<Vec<ProjectEntry>, String> {
    let projects = state.inner.projects.read().await;
    let entries = projects
        .iter()
        .map(|p| ProjectEntry {
            id: p.id.clone(),
            name: p.name.clone(),
            path: p.path.to_string_lossy().to_string(),
            registered_at: None,
        })
        .collect();
    Ok(entries)
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectEntry>, String> {
    list_projects_logic(&state).await
}

// ---------------------------------------------------------------------------
// register_project
// ---------------------------------------------------------------------------

pub(crate) async fn register_project_logic(
    state: &AppState,
    req: RegisterProjectRequest,
) -> Result<ProjectEntry, String> {
    let path = std::path::PathBuf::from(&req.path);

    // Validate that the path exists on disk.
    if !path.exists() {
        return Err(format!("invalid_path: Path does not exist: {}", req.path));
    }

    // Extract the project name from the last directory component.
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| req.path.clone());

    let id = Uuid::new_v4().to_string();

    let info = crate::state::ProjectInfo {
        id: id.clone(),
        path: path.clone(),
        name: name.clone(),
    };

    state.inner.projects.write().await.push(info);

    Ok(ProjectEntry {
        id,
        name,
        path: path.to_string_lossy().to_string(),
        registered_at: None,
    })
}

#[tauri::command]
pub async fn register_project(
    state: State<'_, AppState>,
    req: RegisterProjectRequest,
) -> Result<ProjectEntry, String> {
    register_project_logic(&state, req).await
}

// ---------------------------------------------------------------------------
// unregister_project
// ---------------------------------------------------------------------------

pub(crate) async fn unregister_project_logic(
    state: &AppState,
    id: String,
) -> Result<(), String> {
    let mut projects = state.inner.projects.write().await;
    let original_len = projects.len();
    projects.retain(|p| p.id != id);

    if projects.len() == original_len {
        return Err(format!("not_found: project '{}' not found", id));
    }

    Ok(())
}

#[tauri::command]
pub async fn unregister_project(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    unregister_project_logic(&state, id).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, ProjectInfo};
    use tempfile::tempdir;

    // 1. list_projects_empty_by_default
    #[tokio::test]
    async fn list_projects_empty_by_default() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let result = list_projects_logic(&state).await.unwrap();
        assert!(result.is_empty(), "expected empty list, got: {:?}", result);
    }

    // 2. register_project_adds_to_list
    #[tokio::test]
    async fn register_project_adds_to_list() {
        let dir = tempdir().unwrap();
        let project_dir = dir.path().join("my-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        let state = AppState::new(dir.path().to_path_buf());

        let req = RegisterProjectRequest {
            name: "ignored-name".to_string(),
            path: project_dir.to_string_lossy().to_string(),
        };

        let entry = register_project_logic(&state, req).await.unwrap();

        assert_eq!(entry.name, "my-project");
        assert!(!entry.id.is_empty());
        assert_eq!(entry.path, project_dir.to_string_lossy().to_string());

        // Verify it was added to the state list
        let list = list_projects_logic(&state).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, entry.id);
    }

    // 3. register_project_rejects_nonexistent_path
    #[tokio::test]
    async fn register_project_rejects_nonexistent_path() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let req = RegisterProjectRequest {
            name: "ghost".to_string(),
            path: "/nonexistent/path/that/does/not/exist".to_string(),
        };

        let err = register_project_logic(&state, req).await.unwrap_err();
        assert!(
            err.starts_with("invalid_path:"),
            "expected error starting with 'invalid_path:', got: {}",
            err
        );
    }

    // 4. unregister_project_removes_it
    #[tokio::test]
    async fn unregister_project_removes_it() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        // Manually push a project into state
        {
            let mut projects = state.inner.projects.write().await;
            projects.push(ProjectInfo {
                id: "test-id-123".to_string(),
                path: dir.path().to_path_buf(),
                name: "test-project".to_string(),
            });
        }

        // Verify it's there
        let list = list_projects_logic(&state).await.unwrap();
        assert_eq!(list.len(), 1);

        // Unregister it
        unregister_project_logic(&state, "test-id-123".to_string())
            .await
            .unwrap();

        // Verify it's gone
        let list = list_projects_logic(&state).await.unwrap();
        assert!(list.is_empty(), "expected empty list after unregister");
    }

    // 5. unregister_project_returns_not_found_for_unknown_id
    #[tokio::test]
    async fn unregister_project_returns_not_found_for_unknown_id() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let err = unregister_project_logic(&state, "nonexistent-id".to_string())
            .await
            .unwrap_err();

        assert!(
            err.starts_with("not_found:"),
            "expected error starting with 'not_found:', got: {}",
            err
        );
    }
}
