use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use claude_types::{
    mcp::{AddMcpServerRequest, McpServerInfo},
    plugins::CommandRequest,
};
use serde::Deserialize;
use tokio::process::Command;

use crate::{
    executor::{execute_claude_command, new_request_id},
    state::AppState,
};

// ---------------------------------------------------------------------------
// Query param types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RemoveQuery {
    pub scope: Option<String>,
}

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
// GET /api/v1/mcp/servers
// ---------------------------------------------------------------------------

pub async fn list_mcp_servers(
    Extension(_state): Extension<AppState>,
) -> Json<Vec<McpServerInfo>> {
    // Run `claude mcp list` and capture output.  On failure return empty list.
    let output = Command::new("claude")
        .args(["mcp", "list"])
        .output()
        .await;

    let servers = match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            parse_mcp_list(&text)
        }
        Err(_) => vec![],
    };

    Json(servers)
}

// ---------------------------------------------------------------------------
// POST /api/v1/mcp/servers
// ---------------------------------------------------------------------------

pub async fn add_mcp_server(
    Extension(state): Extension<AppState>,
    Json(body): Json<AddMcpServerRequest>,
) -> Json<CommandRequest> {
    let request_id = new_request_id();
    let rid = request_id.clone();

    tokio::spawn(async move {
        // Build argument list:
        // claude mcp add --transport {transport} --scope {scope}
        //   [-e KEY=VAL]... [-H header]... {name} {commandOrUrl} [args...]
        let scope = body.scope.as_deref().unwrap_or("local").to_string();
        let command_or_url = body.command_or_url.clone().unwrap_or_default();

        let mut args: Vec<String> = vec![
            "mcp".to_string(),
            "add".to_string(),
            "--transport".to_string(),
            body.transport.clone(),
            "--scope".to_string(),
            scope,
        ];

        for (key, val) in &body.env {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, val));
        }

        for (key, val) in &body.headers {
            args.push("-H".to_string());
            args.push(format!("{}: {}", key, val));
        }

        args.push(body.name.clone());
        args.push(command_or_url);

        for arg in &body.args {
            args.push(arg.clone());
        }

        let str_args: Vec<&str> = args.iter().map(String::as_str).collect();
        let _ = execute_claude_command(&state, &str_args, &rid).await;
    });

    Json(CommandRequest { request_id })
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/mcp/servers/:name
// ---------------------------------------------------------------------------

pub async fn remove_mcp_server(
    Extension(state): Extension<AppState>,
    Path(name): Path<String>,
    Query(query): Query<RemoveQuery>,
) -> Json<CommandRequest> {
    let request_id = new_request_id();
    let rid = request_id.clone();
    let scope = query.scope.unwrap_or_else(|| "local".to_string());

    tokio::spawn(async move {
        let args = vec!["mcp", "remove", "--scope", scope.as_str(), name.as_str()];
        let _ = execute_claude_command(&state, &args, &rid).await;
    });

    Json(CommandRequest { request_id })
}
