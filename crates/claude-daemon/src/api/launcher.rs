use axum::{Extension, Json, http::StatusCode};
use claude_types::{ErrorResponse, mcp::LaunchRequest};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// POST /api/v1/launch
// ---------------------------------------------------------------------------

pub async fn launch_claude(
    Extension(_state): Extension<AppState>,
    Json(req): Json<LaunchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let path = std::path::Path::new(&req.project_path);
    if !path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "PATH_NOT_FOUND".to_string(),
                message: format!("Path does not exist: {}", req.project_path),
                validation_errors: vec![],
            }),
        ));
    }

    let mut cmd = tokio::process::Command::new("claude");
    cmd.current_dir(path);
    for (k, v) in &req.env {
        cmd.env(k, v);
    }
    // Detach: don't pipe stdout/stderr, let it run independently
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    cmd.spawn().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "LAUNCH_FAILED".to_string(),
                message: format!("Failed to launch: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(Json(serde_json::json!({
        "status": "launched",
        "projectPath": req.project_path,
    })))
}
