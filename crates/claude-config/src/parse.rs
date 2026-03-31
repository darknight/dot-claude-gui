use std::path::{Path, PathBuf};

use anyhow::Result;
use claude_types::settings::Settings;

/// Reads and parses a `settings.json` file at `path`.
/// Returns `Settings::default()` if the file does not exist.
pub fn read_settings(path: &Path) -> Result<Settings> {
    match std::fs::read_to_string(path) {
        Ok(contents) => parse_settings(&contents),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!("settings file not found at {}, using defaults", path.display());
            Ok(Settings::default())
        }
        Err(e) => Err(anyhow::anyhow!("failed to read settings file {}: {}", path.display(), e)),
    }
}

/// Parses a `Settings` struct from a JSON string.
pub fn parse_settings(json: &str) -> Result<Settings> {
    let settings = serde_json::from_str(json)
        .map_err(|e| anyhow::anyhow!("failed to parse settings JSON: {}", e))?;
    Ok(settings)
}

// ---------------------------------------------------------------------------
// ConfigPaths
// ---------------------------------------------------------------------------

/// Resolves the standard paths for the four configuration layers used by
/// Claude Code:
///
/// | Layer    | Location                                          |
/// |----------|---------------------------------------------------|
/// | managed  | `<claude_home>/settings.json`                     |
/// | user     | `<claude_home>/settings.json` (alias; same file)  |
/// | project  | `<project_dir>/.claude/settings.json`             |
/// | local    | `<project_dir>/.claude/settings.local.json`       |
///
/// When no `project_dir` is supplied the project and local paths are `None`.
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    /// Managed / system-level settings (written by policy tooling).
    pub managed: PathBuf,
    /// Per-user settings stored in the Claude home directory.
    pub user: PathBuf,
    /// Per-project settings committed to the repository.
    pub project: Option<PathBuf>,
    /// Per-project local overrides that are not committed.
    pub local: Option<PathBuf>,
}

impl ConfigPaths {
    pub fn new(claude_home: &Path, project_dir: Option<&Path>) -> Self {
        let managed = claude_home.join("settings.json");
        let user = claude_home.join("settings.json");

        let (project, local) = match project_dir {
            Some(dir) => {
                let dot_claude = dir.join(".claude");
                (
                    Some(dot_claude.join("settings.json")),
                    Some(dot_claude.join("settings.local.json")),
                )
            }
            None => (None, None),
        };

        Self { managed, user, project, local }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn read_nonexistent_returns_default() {
        let path = Path::new("/nonexistent/path/settings.json");
        let result = read_settings(path).expect("should return default for missing file");
        assert_eq!(result, Settings::default());
    }

    #[test]
    fn read_valid_settings_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"{{"language": "en"}}"#).unwrap();
        let settings = read_settings(file.path()).expect("should parse valid settings");
        assert_eq!(settings.language, Some("en".to_string()));
    }

    #[test]
    fn read_invalid_json_returns_error() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "not valid json {{{{").unwrap();
        let result = read_settings(file.path());
        assert!(result.is_err(), "invalid JSON should return an error");
    }

    #[test]
    fn config_paths_without_project() {
        let home = Path::new("/home/user/.claude");
        let paths = ConfigPaths::new(home, None);
        assert_eq!(paths.user, home.join("settings.json"));
        assert_eq!(paths.managed, home.join("settings.json"));
        assert!(paths.project.is_none());
        assert!(paths.local.is_none());
    }

    #[test]
    fn config_paths_with_project() {
        let home = Path::new("/home/user/.claude");
        let project = Path::new("/home/user/myproject");
        let paths = ConfigPaths::new(home, Some(project));
        assert_eq!(
            paths.project.as_deref(),
            Some(Path::new("/home/user/myproject/.claude/settings.json"))
        );
        assert_eq!(
            paths.local.as_deref(),
            Some(Path::new("/home/user/myproject/.claude/settings.local.json"))
        );
    }
}
