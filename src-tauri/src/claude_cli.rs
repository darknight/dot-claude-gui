use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Serialize;

// macOS GUI apps launched from Finder/Dock inherit a minimal PATH
// (`/usr/bin:/bin:/usr/sbin:/sbin`) that does not include the Homebrew prefixes
// or user-local bin dirs where `claude` typically lives. Spawning bare "claude"
// then fails with "No such file or directory (os error 2)" and every plugin /
// marketplace / mcp / launcher command silently fails.
//
// At first use we search a fixed list of well-known install locations, take the
// first hit, and cache it for the lifetime of the process. If nothing is found
// we fall back to the bare command name so `Command::new` still uses any PATH
// the process was given — same behaviour as before this module existed.

static RESOLVED: OnceLock<Resolution> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCliStatus {
    pub resolved: bool,
    pub path: String,
}

#[derive(Debug, Clone)]
struct Resolution {
    resolved: bool,
    path: String,
}

fn detect() -> Resolution {
    let home = std::env::var("HOME").unwrap_or_default();

    let mut candidates: Vec<String> = vec![
        "/opt/homebrew/bin/claude".to_string(),
        "/usr/local/bin/claude".to_string(),
    ];
    if !home.is_empty() {
        for sub in &[
            ".local/bin/claude",
            ".bun/bin/claude",
            ".npm-global/bin/claude",
            ".volta/bin/claude",
            ".cargo/bin/claude",
        ] {
            candidates.push(format!("{}/{}", home, sub));
        }
    }

    for path in &candidates {
        if PathBuf::from(path).exists() {
            return Resolution {
                resolved: true,
                path: path.clone(),
            };
        }
    }

    Resolution {
        resolved: false,
        path: "claude".to_string(),
    }
}

fn resolution() -> &'static Resolution {
    RESOLVED.get_or_init(detect)
}

/// Absolute path to the `claude` CLI if found, else the bare name "claude".
/// Suitable to pass directly to `Command::new` / executor helpers.
pub fn program() -> &'static str {
    resolution().path.as_str()
}

/// Pre-flight status for the frontend banner.
#[tauri::command]
pub fn check_claude_cli() -> Result<ClaudeCliStatus, String> {
    let r = resolution();
    Ok(ClaudeCliStatus {
        resolved: r.resolved,
        path: r.path.clone(),
    })
}
