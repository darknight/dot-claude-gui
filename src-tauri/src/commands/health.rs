use claude_types::HealthResponse;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn health(state: State<'_, AppState>) -> Result<HealthResponse, String> {
    let uptime = state.inner.started_at.elapsed().as_secs();
    Ok(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: Some(uptime),
    })
}
