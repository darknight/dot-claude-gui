use std::process::Stdio;

use claude_types::CommandStream;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::events::{
    CommandCompletedPayload, CommandOutputPayload, EVT_COMMAND_COMPLETED, EVT_COMMAND_OUTPUT,
};

/// Spawn `<program> <args...>` as a subprocess, streaming each stdout/stderr
/// line as a `command-output` Tauri event. Emits `command-completed` when the
/// process exits. Returns the command ID the caller can use to correlate events.
///
/// Guarantees: `command-completed` is ALWAYS the last event for a given
/// `command_id` — both reader tasks drain their pipes before the exit event fires.
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
        .kill_on_drop(true) // ensure orphaned child is killed on error paths
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    let (stdout_done_tx, stdout_done_rx) = oneshot::channel::<()>();
    let (stderr_done_tx, stderr_done_rx) = oneshot::channel::<()>();

    // Stdout streaming task — send completion when loop exits
    {
        let app = app.clone();
        let id = command_id.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            loop {
                match reader.next_line().await {
                    Ok(Some(line)) => {
                        let _ = app.emit(
                            EVT_COMMAND_OUTPUT,
                            CommandOutputPayload {
                                command_id: id.clone(),
                                line,
                                stream: CommandStream::Stdout,
                            },
                        );
                    }
                    Ok(None) => break,
                    Err(e) => {
                        // Surface UTF-8 / read errors as a synthetic stderr line
                        let _ = app.emit(
                            EVT_COMMAND_OUTPUT,
                            CommandOutputPayload {
                                command_id: id.clone(),
                                line: format!("[stdout read error: {}]", e),
                                stream: CommandStream::Stderr,
                            },
                        );
                        break;
                    }
                }
            }
            let _ = stdout_done_tx.send(());
        });
    }

    // Stderr streaming task — mirror structure, send on stderr_done_tx
    {
        let app = app.clone();
        let id = command_id.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            loop {
                match reader.next_line().await {
                    Ok(Some(line)) => {
                        let _ = app.emit(
                            EVT_COMMAND_OUTPUT,
                            CommandOutputPayload {
                                command_id: id.clone(),
                                line,
                                stream: CommandStream::Stderr,
                            },
                        );
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = app.emit(
                            EVT_COMMAND_OUTPUT,
                            CommandOutputPayload {
                                command_id: id.clone(),
                                line: format!("[stderr read error: {}]", e),
                                stream: CommandStream::Stderr,
                            },
                        );
                        break;
                    }
                }
            }
            let _ = stderr_done_tx.send(());
        });
    }

    // Wait-for-exit task — wait for BOTH readers to finish, THEN wait for
    // process, THEN emit completed. If a oneshot is dropped (reader panicked),
    // recv returns Err — proceed anyway.
    {
        let id = command_id.clone();
        tokio::spawn(async move {
            let _ = stdout_done_rx.await;
            let _ = stderr_done_rx.await;

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
