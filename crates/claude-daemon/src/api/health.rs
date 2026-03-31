use axum::{Extension, Json};
use claude_types::HealthResponse;

use crate::state::AppState;

/// `GET /api/v1/health` — returns daemon status, version, and uptime.
pub async fn health_handler(Extension(state): Extension<AppState>) -> Json<HealthResponse> {
    let uptime = state.inner.started_at.elapsed().as_secs();
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: Some(uptime),
    })
}
