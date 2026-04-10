use claude_types::mcp::LaunchRequest;
use serde_json::json;
use std::process::{Command, Stdio};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Launch claude in a detached subprocess at the given project path
// ---------------------------------------------------------------------------

pub(crate) fn launch_claude_logic(req: LaunchRequest) -> Result<serde_json::Value, String> {
    let project_path = std::path::PathBuf::from(&req.project_path);
    if !project_path.exists() {
        return Err(format!("invalid_path: {}", req.project_path));
    }

    let mut cmd = Command::new("claude");
    cmd.current_dir(&project_path);
    for (k, v) in &req.env {
        cmd.env(k, v);
    }

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;

    Ok(json!({
        "status": "launched",
        "projectPath": req.project_path,
    }))
}

#[tauri::command]
pub fn launch_claude(
    _state: State<'_, AppState>,
    req: LaunchRequest,
) -> Result<serde_json::Value, String> {
    launch_claude_logic(req)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn launch_rejects_nonexistent_path() {
        let req = LaunchRequest {
            project_path: "/nonexistent/path/xyz-12345".to_string(),
            env: HashMap::new(),
        };
        let err = launch_claude_logic(req).unwrap_err();
        assert!(err.starts_with("invalid_path:"));
    }
}
