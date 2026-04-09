use std::path::PathBuf;

use axum::{Extension, Json, extract::Path, http::StatusCode};
use claude_types::{
    api::ErrorResponse,
    claudemd::{ClaudeMdFile, ClaudeMdFileDetail, UpdateClaudeMdRequest},
};

use crate::state::AppState;

async fn resolve_claudemd_path(
    state: &AppState,
    id: &str,
) -> Result<PathBuf, (StatusCode, Json<ErrorResponse>)> {
    if id == "global" {
        return Ok(state.inner.claude_home.join("CLAUDE.md"));
    }

    if let Some(project_id) = id.strip_prefix("project:") {
        let projects = state.inner.projects.read().await;
        let project = projects.iter().find(|p| p.id == project_id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "PROJECT_NOT_FOUND".to_string(),
                    message: format!("Project '{}' not found", project_id),
                    validation_errors: vec![],
                }),
            )
        })?;
        return Ok(project.path.join(".claude").join("CLAUDE.md"));
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            code: "INVALID_ID".to_string(),
            message: format!("Invalid CLAUDE.md id format: '{}'", id),
            validation_errors: vec![],
        }),
    ))
}

pub async fn list_claudemd_files(
    Extension(state): Extension<AppState>,
) -> Json<Vec<ClaudeMdFile>> {
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
        let project_path = project.path.join(".claude").join("CLAUDE.md");
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

    Json(result)
}

pub async fn get_claudemd_file(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClaudeMdFileDetail>, (StatusCode, Json<ErrorResponse>)> {
    let file_path = resolve_claudemd_path(&state, &id).await?;

    let content = std::fs::read_to_string(&file_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: format!("CLAUDE.md not found at '{}'", file_path.display()),
                validation_errors: vec![],
            }),
        )
    })?;

    let scope = if id == "global" { "global" } else { "project" };
    let project_id = id.strip_prefix("project:").map(|s| s.to_string());

    Ok(Json(ClaudeMdFileDetail {
        id,
        scope: scope.to_string(),
        filename: "CLAUDE.md".to_string(),
        path: file_path.to_string_lossy().into_owned(),
        content,
        project_id,
    }))
}

pub async fn put_claudemd_file(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateClaudeMdRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let file_path = resolve_claudemd_path(&state, &id).await?;

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "DIR_ERROR".to_string(),
                    message: format!("Failed to create directory: {}", e),
                    validation_errors: vec![],
                }),
            )
        })?;
    }

    claude_config::write::atomic_write(&file_path, body.content.as_bytes()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "WRITE_ERROR".to_string(),
                message: format!("Failed to write CLAUDE.md: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(StatusCode::OK)
}

pub async fn delete_claudemd_file(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let file_path = resolve_claudemd_path(&state, &id).await?;

    std::fs::remove_file(&file_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: format!("CLAUDE.md not found at '{}'", file_path.display()),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
