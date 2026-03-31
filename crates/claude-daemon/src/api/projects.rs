use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
};
use claude_types::api::{ErrorResponse, ProjectEntry, RegisterProjectRequest};
use uuid::Uuid;

use crate::state::{AppState, ProjectInfo};

// ---------------------------------------------------------------------------
// GET /api/v1/projects
// ---------------------------------------------------------------------------

/// Returns the list of all registered projects.
pub async fn list_projects(
    Extension(state): Extension<AppState>,
) -> Json<Vec<ProjectEntry>> {
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
    Json(entries)
}

// ---------------------------------------------------------------------------
// POST /api/v1/projects
// ---------------------------------------------------------------------------

/// Registers a new project by validating its path and adding it to the list.
pub async fn register_project(
    Extension(state): Extension<AppState>,
    Json(body): Json<RegisterProjectRequest>,
) -> Result<(StatusCode, Json<ProjectEntry>), (StatusCode, Json<ErrorResponse>)> {
    let path = std::path::PathBuf::from(&body.path);

    // Validate that the path exists on disk.
    if !path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "INVALID_PATH".to_string(),
                message: format!("Path does not exist: {}", body.path),
                validation_errors: vec![],
            }),
        ));
    }

    // Extract the project name from the last directory component.
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| body.path.clone());

    let id = Uuid::new_v4().to_string();

    let info = ProjectInfo {
        id: id.clone(),
        path: path.clone(),
        name: name.clone(),
    };

    state.inner.projects.write().await.push(info);

    let entry = ProjectEntry {
        id,
        name,
        path: path.to_string_lossy().to_string(),
        registered_at: None,
    };

    Ok((StatusCode::CREATED, Json(entry)))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/projects/{id}
// ---------------------------------------------------------------------------

/// Unregisters a project by ID.
pub async fn delete_project(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut projects = state.inner.projects.write().await;
    let original_len = projects.len();
    projects.retain(|p| p.id != id);

    if projects.len() == original_len {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "PROJECT_NOT_FOUND".to_string(),
                message: format!("Project '{}' not found", id),
                validation_errors: vec![],
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
