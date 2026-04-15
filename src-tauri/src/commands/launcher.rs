use claude_types::mcp::LaunchRequest;
use serde_json::json;
use std::process::Command;
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Launch `claude` inside a new terminal window so its TUI is visible.
// macOS: uses AppleScript to open Terminal.app with the requested cwd and env.
// Other platforms: TODO — for now, returns an error.
// ---------------------------------------------------------------------------

pub(crate) fn launch_claude_logic(req: LaunchRequest) -> Result<serde_json::Value, String> {
    let project_path = std::path::PathBuf::from(&req.project_path);
    if !project_path.exists() {
        return Err(format!("invalid_path: {}", req.project_path));
    }

    #[cfg(target_os = "macos")]
    {
        // Build `KEY='VAL' KEY2='VAL2' ...` env prefix, escaping single quotes.
        let env_prefix = req
            .env
            .iter()
            .map(|(k, v)| format!("{}='{}'", k, v.replace('\'', "'\\''")))
            .collect::<Vec<_>>()
            .join(" ");

        let path_escaped = req.project_path.replace('\'', "'\\''");
        let shell_cmd = if env_prefix.is_empty() {
            format!("cd '{}' && claude", path_escaped)
        } else {
            format!("cd '{}' && {} claude", path_escaped, env_prefix)
        };

        // AppleScript needs `\` and `"` inside the do script string escaped.
        let script_arg = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");
        let osa_script = format!(
            "tell application \"Terminal\"\n  activate\n  do script \"{}\"\nend tell",
            script_arg
        );

        Command::new("osascript")
            .args(["-e", &osa_script])
            .spawn()
            .map_err(|e| format!("spawn osascript: {e}"))?;

        return Ok(json!({
            "status": "launched",
            "projectPath": req.project_path,
            "terminal": "Terminal.app",
        }));
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("launch_unsupported: only macOS is supported for now".to_string())
    }
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
