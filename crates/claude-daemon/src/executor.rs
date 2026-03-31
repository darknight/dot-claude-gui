use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

use claude_types::{CommandStream, WsEvent};

use crate::state::AppState;

/// The collected result of a completed command.
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Generate a unique request ID.
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Execute a `claude` CLI command, streaming each line of output via WebSocket.
///
/// Both stdout and stderr are read concurrently with `tokio::select!`.  Each
/// line is broadcast as a `WsEvent::CommandOutput` message.  When the process
/// exits a `WsEvent::CommandCompleted` message is broadcast and a
/// [`CommandResult`] containing the full accumulated output is returned.
pub async fn execute_claude_command(
    state: &AppState,
    args: &[&str],
    request_id: &str,
) -> Result<CommandResult> {
    let mut child = Command::new("claude")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture stderr"))?;

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    let mut stdout_buf = String::new();
    let mut stderr_buf = String::new();

    // Drain both streams concurrently until both are exhausted.
    let mut stdout_done = false;
    let mut stderr_done = false;

    loop {
        if stdout_done && stderr_done {
            break;
        }

        tokio::select! {
            // Read next stdout line
            line = stdout_lines.next_line(), if !stdout_done => {
                match line? {
                    Some(data) => {
                        stdout_buf.push_str(&data);
                        stdout_buf.push('\n');
                        state.broadcast(WsEvent::CommandOutput {
                            command_id: request_id.to_string(),
                            line: data,
                            stream: CommandStream::Stdout,
                        });
                    }
                    None => {
                        stdout_done = true;
                    }
                }
            }

            // Read next stderr line
            line = stderr_lines.next_line(), if !stderr_done => {
                match line? {
                    Some(data) => {
                        stderr_buf.push_str(&data);
                        stderr_buf.push('\n');
                        state.broadcast(WsEvent::CommandOutput {
                            command_id: request_id.to_string(),
                            line: data,
                            stream: CommandStream::Stderr,
                        });
                    }
                    None => {
                        stderr_done = true;
                    }
                }
            }
        }
    }

    let status = child.wait().await?;
    let exit_code = status.code().unwrap_or(-1);

    state.broadcast(WsEvent::CommandCompleted {
        command_id: request_id.to_string(),
        exit_code,
    });

    Ok(CommandResult {
        exit_code,
        stdout: stdout_buf,
        stderr: stderr_buf,
    })
}
