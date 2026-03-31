use std::path::Path;

use anyhow::Result;
use claude_types::settings::Settings;

/// Serializes `settings` to pretty-printed JSON and atomically writes it to
/// `path`.
pub fn write_settings(path: &Path, settings: &Settings) -> Result<()> {
    let json = serde_json::to_vec_pretty(settings)
        .map_err(|e| anyhow::anyhow!("failed to serialize settings: {}", e))?;
    atomic_write(path, &json)
}

/// Atomically writes `data` to `path` by first writing to a sibling temp file
/// then renaming it into place. Creates parent directories if they do not
/// exist.
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("failed to create parent dirs for {}: {}", path.display(), e))?;
    }

    // Build a temporary file name in the same directory so the rename is
    // on the same filesystem (required for an atomic rename on most OSes).
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("path has no file name: {}", path.display()))?
        .to_string_lossy();
    let tmp_path = path.with_file_name(format!(".{}.tmp", file_name));

    std::fs::write(&tmp_path, data)
        .map_err(|e| anyhow::anyhow!("failed to write temp file {}: {}", tmp_path.display(), e))?;

    std::fs::rename(&tmp_path, path)
        .map_err(|e| {
            // Best-effort cleanup on rename failure.
            let _ = std::fs::remove_file(&tmp_path);
            anyhow::anyhow!("failed to rename {} -> {}: {}", tmp_path.display(), path.display(), e)
        })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::read_settings;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let data = b"hello world";
        atomic_write(&path, data).expect("write should succeed");
        let contents = std::fs::read(&path).unwrap();
        assert_eq!(contents, data);
    }

    #[test]
    fn atomic_write_no_temp_file_left() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        atomic_write(&path, b"{}").expect("write should succeed");

        // The temp file should not be present after a successful write.
        let tmp = path.with_file_name(".settings.json.tmp");
        assert!(!tmp.exists(), "temp file should not exist after atomic write");
    }

    #[test]
    fn write_settings_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let mut original = Settings::default();
        original.language = Some("fr".to_string());
        original.include_co_authored_by = Some(false);

        write_settings(&path, &original).expect("write should succeed");
        let loaded = read_settings(&path).expect("read should succeed");

        assert_eq!(original, loaded);
    }

    #[test]
    fn atomic_write_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("settings.json");
        atomic_write(&path, b"{}").expect("write should create parent dirs");
        assert!(path.exists());
    }
}
