use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tracing::warn;

/// Describes what kind of change occurred to a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChangeKind {
    Created,
    Modified,
    Removed,
}

/// Represents a file change event with the affected path and kind of change.
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub kind: FileChangeKind,
}

/// Returns true if the path has a `.json` or `.md` extension.
fn is_relevant_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") | Some("md") => true,
        _ => false,
    }
}

/// Map a notify `EventKind` to a `FileChangeKind`, returning `None` for uninteresting events.
fn map_event_kind(kind: &EventKind) -> Option<FileChangeKind> {
    match kind {
        EventKind::Create(_) => Some(FileChangeKind::Created),
        EventKind::Modify(_) => Some(FileChangeKind::Modified),
        EventKind::Remove(_) => Some(FileChangeKind::Removed),
        _ => None,
    }
}

/// Watch the given directories and emit `FileChangeEvent`s for `.json` and `.md` files.
///
/// Paths that do not exist are skipped with a warning rather than causing an error.
/// Returns a `(RecommendedWatcher, Receiver<FileChangeEvent>)` pair.
pub fn watch_directories(
    paths: &[PathBuf],
) -> Result<(RecommendedWatcher, mpsc::Receiver<FileChangeEvent>)> {
    let (event_tx, event_rx) = mpsc::channel::<FileChangeEvent>();

    let config = Config::default().with_poll_interval(Duration::from_secs(2));

    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if let Some(change_kind) = map_event_kind(&event.kind) {
                    for path in &event.paths {
                        if is_relevant_file(path) {
                            let _ = event_tx.send(FileChangeEvent {
                                path: path.clone(),
                                kind: change_kind.clone(),
                            });
                        }
                    }
                }
            }
        },
        config,
    )?;

    for path in paths {
        if !path.exists() {
            warn!("Watch path does not exist, skipping: {}", path.display());
            continue;
        }
        watcher.watch(path, RecursiveMode::Recursive)?;
    }

    Ok((watcher, event_rx))
}

/// Add a path to an existing watcher.
pub fn add_watch_path(watcher: &mut RecommendedWatcher, path: &Path) -> Result<()> {
    watcher.watch(path, RecursiveMode::Recursive)?;
    Ok(())
}

/// Remove a path from an existing watcher.
pub fn remove_watch_path(watcher: &mut RecommendedWatcher, path: &Path) -> Result<()> {
    watcher.unwatch(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn watch_detects_json_file_creation() {
        let dir = TempDir::new().unwrap();
        // Canonicalize so that symlink-resolved paths (e.g. /private/var on macOS) match.
        let real_dir = dir.path().canonicalize().unwrap();
        let paths = vec![real_dir.clone()];

        let (_watcher, rx) = watch_directories(&paths).unwrap();

        // Give the watcher time to initialize.
        std::thread::sleep(Duration::from_millis(200));

        let json_path = real_dir.join("test.json");
        fs::write(&json_path, r#"{"key": "value"}"#).unwrap();

        // Drain until we see a Created event for our file (there may be other events first).
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                panic!("expected a Created FileChangeEvent within 5 seconds");
            }
            let event = rx.recv_timeout(remaining).expect("expected a FileChangeEvent within 5 seconds");
            if event.path == json_path && event.kind == FileChangeKind::Created {
                break;
            }
        }
    }

    #[test]
    fn watch_detects_file_modification() {
        let dir = TempDir::new().unwrap();
        // Canonicalize so that symlink-resolved paths (e.g. /private/var on macOS) match.
        let real_dir = dir.path().canonicalize().unwrap();
        let json_path = real_dir.join("config.json");
        fs::write(&json_path, r#"{"initial": true}"#).unwrap();

        let paths = vec![real_dir.clone()];
        let (_watcher, rx) = watch_directories(&paths).unwrap();

        // Give the watcher time to initialize.
        std::thread::sleep(Duration::from_millis(200));

        fs::write(&json_path, r#"{"modified": true}"#).unwrap();

        // Drain until we see a Modified (or Created on some OS) event for our file.
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                panic!("expected a Modified FileChangeEvent within 5 seconds");
            }
            let event = rx.recv_timeout(remaining).expect("expected a FileChangeEvent within 5 seconds");
            if event.path == json_path && event.kind == FileChangeKind::Modified {
                break;
            }
        }
    }

    #[test]
    fn watch_ignores_non_json_md_files() {
        let dir = TempDir::new().unwrap();
        let paths = vec![dir.path().to_path_buf()];

        let (_watcher, rx) = watch_directories(&paths).unwrap();

        // Give the watcher time to initialize.
        std::thread::sleep(Duration::from_millis(200));

        let txt_path = dir.path().join("ignored.txt");
        fs::write(&txt_path, "some text").unwrap();

        // We should receive NO event within 2 seconds.
        match rx.recv_timeout(Duration::from_secs(2)) {
            Err(mpsc::RecvTimeoutError::Timeout) => { /* expected */ }
            Ok(event) => panic!("unexpected event for .txt file: {:?}", event),
            Err(e) => panic!("unexpected channel error: {:?}", e),
        }
    }

    #[test]
    fn watch_nonexistent_path_does_not_error() {
        let nonexistent = PathBuf::from("/tmp/dot-claude-gui-test-nonexistent-12345");
        let paths = vec![nonexistent];
        // Should succeed without error — nonexistent paths are skipped with a warning.
        let result = watch_directories(&paths);
        assert!(result.is_ok(), "watch_directories should not error on nonexistent paths");
    }
}
