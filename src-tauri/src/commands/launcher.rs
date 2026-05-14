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

pub(crate) fn build_launch_env(
    home: &std::path::Path,
    account: Option<&str>,
    user_env: &std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, String> {
    let mut env = user_env.clone();
    match account {
        Some(name) if name != "default" && !env.contains_key("CLAUDE_CONFIG_DIR") => {
            let dir = crate::app_config::account_dir(home, name);
            env.insert(
                "CLAUDE_CONFIG_DIR".to_string(),
                dir.to_string_lossy().into_owned(),
            );
        }
        _ => {}
    }
    env
}

pub(crate) fn launch_claude_logic(req: LaunchRequest) -> Result<serde_json::Value, String> {
    if let Some(p) = &req.project_path {
        if !std::path::PathBuf::from(p).exists() {
            return Err(format!("invalid_path: {}", p));
        }
    }

    let home = dirs_next::home_dir().ok_or("cannot determine home directory")?;
    let resolved_env = build_launch_env(&home, req.account.as_deref(), &req.env);

    #[cfg(target_os = "macos")]
    {
        let terminal = req.preferred_terminal.as_deref().unwrap_or("terminal");
        let osa_script = build_osa_script(terminal, req.project_path.as_deref(), &resolved_env, &req.args);

        let output = Command::new("osascript")
            .args(["-e", &osa_script])
            .output()
            .map_err(|e| format!("spawn osascript: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("launch_failed: {}", stderr.trim()));
        }

        let terminal_name = match terminal {
            "iterm2" => "iTerm",
            _ => "Terminal.app",
        };
        return Ok(json!({
            "status": "launched",
            "projectPath": req.project_path,
            "terminal": terminal_name,
        }));
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = req;
        let _ = resolved_env;
        Err("launch_unsupported: only macOS is supported for now".to_string())
    }
}

#[cfg(target_os = "macos")]
fn build_osa_script(
    terminal: &str,
    project_path: Option<&str>,
    env: &std::collections::HashMap<String, String>,
    args: &[String],
) -> String {
    // Build `KEY='VAL' KEY2='VAL2' ...` env prefix, escaping single quotes.
    let env_prefix = env
        .iter()
        .map(|(k, v)| format!("{}='{}'", k, v.replace('\'', "'\\''")))
        .collect::<Vec<_>>()
        .join(" ");

    // Append CLI args, each single-quote-escaped.
    let args_suffix = if args.is_empty() {
        String::new()
    } else {
        let escaped: Vec<String> = args
            .iter()
            .map(|a| format!("'{}'", a.replace('\'', "'\\''")))
            .collect();
        format!(" {}", escaped.join(" "))
    };

    let cd_prefix = match project_path {
        Some(p) => format!("cd '{}' && ", p.replace('\'', "'\\''")),
        None => String::new(),
    };
    let shell_cmd = if env_prefix.is_empty() {
        format!("{}claude{}", cd_prefix, args_suffix)
    } else {
        format!("{}export {} && claude{}", cd_prefix, env_prefix, args_suffix)
    };

    // AppleScript needs `\` and `"` inside string literals escaped.
    let script_arg = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");

    match terminal {
        "iterm2" => format!(
            "tell application \"iTerm\"\n  activate\n  create window with default profile\n  tell current session of current window to write text \"{}\"\nend tell",
            script_arg
        ),
        _ => format!(
            "tell application \"Terminal\"\n  activate\n  do script \"{}\"\nend tell",
            script_arg
        ),
    }
}

#[tauri::command]
pub fn launch_claude(
    _state: State<'_, AppState>,
    req: LaunchRequest,
) -> Result<serde_json::Value, String> {
    launch_claude_logic(req)
}

#[tauri::command]
pub fn get_claude_args() -> Result<String, String> {
    let output = Command::new(crate::claude_cli::program())
        .arg("--help")
        .output()
        .map_err(|e| format!("spawn_claude: {e}"))?;
    if !output.status.success() {
        return Err(format!("claude_help_failed: exit {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
            project_path: Some("/nonexistent/path/xyz-12345".to_string()),
            env: HashMap::new(),
            args: vec![],
            preferred_terminal: None,
            account: None,
        };
        let err = launch_claude_logic(req).unwrap_err();
        assert!(err.starts_with("invalid_path:"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_defaults_to_terminal_app() {
        let env = HashMap::new();
        let script = build_osa_script("terminal", Some("/tmp"), &env, &[]);
        assert!(script.contains("tell application \"Terminal\""));
        assert!(script.contains("do script"));
        assert!(script.contains("cd '/tmp' && claude"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_uses_iterm_for_iterm2() {
        let env = HashMap::new();
        let script = build_osa_script("iterm2", Some("/tmp"), &env, &[]);
        assert!(script.contains("tell application \"iTerm\""));
        assert!(script.contains("create window with default profile"));
        assert!(script.contains("write text"));
        assert!(!script.contains("tell application \"Terminal\""));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_unknown_terminal_falls_back_to_terminal_app() {
        let env = HashMap::new();
        let script = build_osa_script("warp", Some("/tmp"), &env, &[]);
        assert!(script.contains("tell application \"Terminal\""));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_includes_env_prefix() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        let script = build_osa_script("iterm2", Some("/tmp"), &env, &[]);
        assert!(script.contains("FOO='bar'"));
        assert!(script.contains("export FOO='bar' && claude"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_escapes_single_quotes_in_path() {
        let env = HashMap::new();
        let script = build_osa_script("terminal", Some("/tmp/it's"), &env, &[]);
        // After shell-escape the path becomes  /tmp/it'\''s  (10 chars).
        // After AppleScript-escape (\\ -> \\\\) it becomes  /tmp/it'\\''s  (11 chars).
        // In a Rust string literal that's "/tmp/it'\\\\''s".
        assert!(script.contains("/tmp/it'\\\\''s"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_appends_args() {
        let env = HashMap::new();
        let args = vec![
            "--effort".to_string(),
            "high".to_string(),
            "--brief".to_string(),
        ];
        let script = build_osa_script("terminal", Some("/tmp"), &env, &args);
        assert!(script.contains("claude '--effort' 'high' '--brief'"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_escapes_single_quotes_in_args() {
        let env = HashMap::new();
        let args = vec!["--name".to_string(), "it's".to_string()];
        let script = build_osa_script("terminal", Some("/tmp"), &env, &args);
        // it's -> it'\''s, then AppleScript escaped (\\ doubled) -> it'\\''s
        assert!(script.contains("'it'\\\\''s'"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn osa_script_omits_cd_when_no_project_path() {
        let mut env = HashMap::new();
        env.insert("CLAUDE_CONFIG_DIR".to_string(), "/foo".to_string());
        let script = build_osa_script("terminal", None, &env, &["/login".to_string()]);
        assert!(!script.contains("cd "));
        assert!(script.contains("export CLAUDE_CONFIG_DIR='/foo' && claude '/login'"));
    }

    #[test]
    fn build_env_for_default_account_omits_claude_config_dir() {
        let home = std::path::PathBuf::from("/home/u");
        let env = build_launch_env(&home, Some("default"), &HashMap::new());
        assert!(!env.contains_key("CLAUDE_CONFIG_DIR"));
    }

    #[test]
    fn build_env_for_named_account_injects_claude_config_dir() {
        let home = std::path::PathBuf::from("/home/u");
        let env = build_launch_env(&home, Some("work"), &HashMap::new());
        assert_eq!(
            env.get("CLAUDE_CONFIG_DIR").map(String::as_str),
            Some("/home/u/.dot-claude-gui/accounts/work")
        );
    }

    #[test]
    fn build_env_user_override_wins_for_claude_config_dir() {
        let home = std::path::PathBuf::from("/home/u");
        let mut user = HashMap::new();
        user.insert("CLAUDE_CONFIG_DIR".to_string(), "/custom".to_string());
        let env = build_launch_env(&home, Some("work"), &user);
        assert_eq!(env.get("CLAUDE_CONFIG_DIR").map(String::as_str), Some("/custom"));
    }

    #[test]
    fn build_env_nil_account_omits_injection() {
        let home = std::path::PathBuf::from("/home/u");
        let env = build_launch_env(&home, None, &HashMap::new());
        assert!(!env.contains_key("CLAUDE_CONFIG_DIR"));
    }
}
