use std::process::Stdio;

use claude_types::CommandStream;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

use crate::events::{
    CommandCompletedPayload, CommandOutputPayload, EVT_COMMAND_COMPLETED, EVT_COMMAND_OUTPUT,
};

/// Spawn `<program> <args...>` as a subprocess, streaming each stdout/stderr
/// line as a `command-output` Tauri event. Emits `command-completed` when the
/// process exits. Returns the command ID the caller can use to correlate events.
pub fn spawn_streaming(
    app: AppHandle,
    program: &str,
    args: Vec<String>,
) -> Result<String, String> {
    let command_id = Uuid::new_v4().to_string();

    let mut child = Command::new(program)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    // Stdout streaming task
    {
        let app = app.clone();
        let id = command_id.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app.emit(
                    EVT_COMMAND_OUTPUT,
                    CommandOutputPayload {
                        command_id: id.clone(),
                        line,
                        stream: CommandStream::Stdout,
                    },
                );
            }
        });
    }

    // Stderr streaming task
    {
        let app = app.clone();
        let id = command_id.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app.emit(
                    EVT_COMMAND_OUTPUT,
                    CommandOutputPayload {
                        command_id: id.clone(),
                        line,
                        stream: CommandStream::Stderr,
                    },
                );
            }
        });
    }

    // Wait-for-exit task
    {
        let id = command_id.clone();
        tokio::spawn(async move {
            let exit_code = child
                .wait()
                .await
                .map(|s| s.code().unwrap_or(-1))
                .unwrap_or(-1);
            let _ = app.emit(
                EVT_COMMAND_COMPLETED,
                CommandCompletedPayload {
                    command_id: id,
                    exit_code,
                },
            );
        });
    }

    Ok(command_id)
}
