use std::process::Command;
use tauri::{AppHandle, State};

use claude_types::{
    mcp::{AddMcpServerRequest, McpServerInfo},
    plugins::CommandRequest,
};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse the output of `claude mcp list` into a list of `McpServerInfo`.
///
/// Each server line has the form:
///   `{name}: {command_or_url} - {status}`
///
/// Lines that don't contain both `: ` and ` - ` are skipped (they are
/// headers, blank lines, etc.).
fn parse_mcp_list(output: &str) -> Vec<McpServerInfo> {
    let mut servers = Vec::new();

    for line in output.lines() {
        // Must contain `: ` and ` - ` to be a server line.
        if !line.contains(": ") || !line.contains(" - ") {
            continue;
        }

        // Split on first `: ` to get name (trim leading whitespace / bullet chars).
        let (raw_name, rest) = match line.split_once(": ") {
            Some(parts) => parts,
            None => continue,
        };

        let name = raw_name.trim().trim_start_matches(['❯', '!', ' ']).trim().to_string();
        if name.is_empty() {
            continue;
        }

        // Split on *last* ` - ` to get command/url and status.
        let (command_or_url, status) = match rest.rfind(" - ") {
            Some(pos) => (&rest[..pos], &rest[pos + 3..]),
            None => continue,
        };

        let command_or_url = command_or_url.trim().to_string();
        let status = status.trim().to_string();

        // Determine transport and set command vs url fields.
        let (transport, command, url) =
            if command_or_url.starts_with("http://") || command_or_url.starts_with("https://") {
                ("http".to_string(), None, Some(command_or_url))
            } else {
                ("stdio".to_string(), Some(command_or_url), None)
            };

        servers.push(McpServerInfo {
            name,
            scope: "unknown".to_string(),
            transport,
            command,
            args: vec![],
            url,
            env: Default::default(),
            headers: Default::default(),
            status: if status.is_empty() { None } else { Some(status) },
        });
    }

    servers
}

// ---------------------------------------------------------------------------
// list_mcp_servers — SYNC, full implementation
// ---------------------------------------------------------------------------

pub(crate) fn list_mcp_servers_logic() -> Vec<McpServerInfo> {
    let output = Command::new("claude")
        .args(["mcp", "list"])
        .output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            parse_mcp_list(&text)
        }
        Err(_) => vec![],
    }
}

#[tauri::command]
pub fn list_mcp_servers(_state: State<'_, AppState>) -> Result<Vec<McpServerInfo>, String> {
    Ok(list_mcp_servers_logic())
}

// ---------------------------------------------------------------------------
// add_mcp_server — streams `claude mcp add` via executor
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn add_mcp_server(
    app: AppHandle,
    _state: State<'_, AppState>,
    req: AddMcpServerRequest,
) -> Result<CommandRequest, String> {
    let args = build_mcp_add_args(&req);
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

fn build_mcp_add_args(req: &AddMcpServerRequest) -> Vec<String> {
    // Mirror daemon's add_mcp_server arg construction exactly:
    // claude mcp add --transport <transport> --scope <scope>
    //   [-e KEY=VAL]... [-H key: val]... <name> <commandOrUrl> [args...]
    let scope = req.scope.as_deref().unwrap_or("local").to_string();
    let command_or_url = req.command_or_url.clone().unwrap_or_default();

    let mut args: Vec<String> = vec![
        "mcp".to_string(),
        "add".to_string(),
        "--transport".to_string(),
        req.transport.clone(),
        "--scope".to_string(),
        scope,
    ];

    for (key, val) in &req.env {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, val));
    }

    for (key, val) in &req.headers {
        args.push("-H".to_string());
        args.push(format!("{}: {}", key, val));
    }

    args.push(req.name.clone());
    args.push(command_or_url);

    for arg in &req.args {
        args.push(arg.clone());
    }

    args
}

// ---------------------------------------------------------------------------
// remove_mcp_server — streams `claude mcp remove` via executor
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn remove_mcp_server(
    app: AppHandle,
    _state: State<'_, AppState>,
    name: String,
    scope: Option<String>,
) -> Result<CommandRequest, String> {
    // Mirror daemon: claude mcp remove --scope <scope> <name>
    let scope_val = scope.unwrap_or_else(|| "local".to_string());
    let args = vec![
        "mcp".to_string(),
        "remove".to_string(),
        "--scope".to_string(),
        scope_val,
        name,
    ];
    let request_id = crate::executor::spawn_streaming(app, "claude", args)?;
    Ok(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mcp_list_handles_empty() {
        let servers = parse_mcp_list("");
        assert!(servers.is_empty());
    }

    #[test]
    fn parse_mcp_list_handles_blank_lines_and_headers() {
        let input = "\nMCP Servers:\n\n  No servers configured.\n";
        let servers = parse_mcp_list(input);
        assert!(servers.is_empty());
    }

    #[test]
    fn parse_mcp_list_parses_stdio_server() {
        let input = "  my-server: npx my-mcp-tool - connected\n";
        let servers = parse_mcp_list(input);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "my-server");
        assert_eq!(servers[0].transport, "stdio");
        assert_eq!(servers[0].command, Some("npx my-mcp-tool".to_string()));
        assert_eq!(servers[0].url, None);
        assert_eq!(servers[0].status, Some("connected".to_string()));
    }

    #[test]
    fn parse_mcp_list_parses_http_server() {
        let input = "  remote: https://example.com/mcp - disconnected\n";
        let servers = parse_mcp_list(input);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "remote");
        assert_eq!(servers[0].transport, "http");
        assert_eq!(servers[0].command, None);
        assert_eq!(servers[0].url, Some("https://example.com/mcp".to_string()));
        assert_eq!(servers[0].status, Some("disconnected".to_string()));
    }

    #[test]
    fn parse_mcp_list_handles_bullet_prefix() {
        // CLI sometimes outputs `❯ name: cmd - status`
        let input = "❯ tools-server: /usr/local/bin/mcp-tools - running\n";
        let servers = parse_mcp_list(input);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "tools-server");
    }

    #[test]
    fn parse_mcp_list_uses_last_dash_separator_for_status() {
        // command itself contains ` - ` — status should be taken from the LAST occurrence
        let input = "  tricky: node /path - to/tool - ok\n";
        let servers = parse_mcp_list(input);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].command, Some("node /path - to/tool".to_string()));
        assert_eq!(servers[0].status, Some("ok".to_string()));
    }

    #[test]
    fn parse_mcp_list_parses_multiple_servers() {
        let input = "\
  server-a: npx tool-a - connected
  server-b: https://b.example.com - error
  server-c: /usr/bin/mcp-c - connected
";
        let servers = parse_mcp_list(input);
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[0].name, "server-a");
        assert_eq!(servers[1].name, "server-b");
        assert_eq!(servers[2].name, "server-c");
    }

    #[test]
    fn parse_mcp_list_skips_lines_without_required_separators() {
        let input = "  just a line without separators\n  name: only-colon-no-dash\n";
        let servers = parse_mcp_list(input);
        assert!(servers.is_empty());
    }
}
