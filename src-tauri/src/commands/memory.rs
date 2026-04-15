use claude_types::memory::{MemoryFile, MemoryFileDetail, MemoryProject};
use tauri::State;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Frontmatter parsing
// ---------------------------------------------------------------------------

struct MemoryFrontmatter {
    name: Option<String>,
    description: Option<String>,
    memory_type: Option<String>,
}

/// Parse YAML frontmatter from a memory file's contents.
///
/// Valid frontmatter block:
/// - Line 0 must be `---`
/// - Subsequent lines are scanned for `name:`, `description:`, and `type:`
///   until the closing `---` is found.
fn parse_frontmatter(contents: &str) -> Option<MemoryFrontmatter> {
    let mut lines = contents.lines();

    if lines.next()?.trim() != "---" {
        return None;
    }

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut memory_type: Option<String> = None;
    let mut closed = false;

    for line in lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        if let Some(rest) = line.strip_prefix("name:") {
            name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("description:") {
            description = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("type:") {
            memory_type = Some(rest.trim().to_string());
        }
    }

    if !closed {
        return None;
    }

    Some(MemoryFrontmatter { name, description, memory_type })
}

// ---------------------------------------------------------------------------
// Path decode helper
// ---------------------------------------------------------------------------

/// Decode a project directory name into a human-readable path.
///
/// Directory names encode paths by replacing `/` with `-`. This encoding is
/// ambiguous: `whoishiring-insight` (literal dash) and `whoishiring/insight`
/// (path separator) both encode to the same string.
///
/// To resolve this ambiguity, we try to read a session JSONL file inside the
/// project directory and extract the `cwd` field, which contains the original
/// path unambiguously. Falls back to naive `-` → `/` replacement otherwise.
fn decode_project_path(project_id: &str, project_dir: &std::path::Path) -> String {
    if let Some(cwd) = read_cwd_from_sessions(project_dir) {
        return cwd;
    }
    project_id.replace('-', "/")
}

/// Search the project directory for any JSONL session file and extract the
/// `cwd` field from its first line. Returns `None` if no JSONL file with a
/// parseable `cwd` is found.
fn read_cwd_from_sessions(project_dir: &std::path::Path) -> Option<String> {
    // Walk up to 2 levels deep looking for any .jsonl file
    fn find_jsonl(dir: &std::path::Path, depth: usize) -> Option<std::path::PathBuf> {
        if depth > 2 {
            return None;
        }
        let entries = std::fs::read_dir(dir).ok()?;
        // First pass: look for jsonl files at this level
        let mut subdirs = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                return Some(path);
            }
            if path.is_dir() {
                subdirs.push(path);
            }
        }
        // Second pass: recurse into subdirectories
        for subdir in subdirs {
            if let Some(p) = find_jsonl(&subdir, depth + 1) {
                return Some(p);
            }
        }
        None
    }

    let jsonl_path = find_jsonl(project_dir, 0)?;
    let contents = std::fs::read_to_string(&jsonl_path).ok()?;
    let first_line = contents.lines().next()?;
    let json: serde_json::Value = serde_json::from_str(first_line).ok()?;
    json.get("cwd")?.as_str().map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// Logic helpers (testable without Tauri State)
// ---------------------------------------------------------------------------

pub(crate) fn list_memory_projects_logic(state: &AppState) -> Vec<MemoryProject> {
    let projects_dir = state.inner.claude_home.join("projects");

    let read_dir = match std::fs::read_dir(&projects_dir) {
        Ok(rd) => rd,
        Err(_) => return vec![],
    };

    let mut result = Vec::new();

    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let id = match entry_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let memory_dir = entry_path.join("memory");
        if !memory_dir.is_dir() {
            continue;
        }

        // Count .md files in the memory directory.
        let file_count = std::fs::read_dir(&memory_dir)
            .map(|rd| {
                rd.flatten()
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "md")
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);

        if file_count == 0 {
            continue;
        }

        result.push(MemoryProject {
            project_path: decode_project_path(&id, &entry_path),
            id,
            file_count,
        });
    }

    result
}

pub(crate) fn list_memory_files_logic(
    state: &AppState,
    project_id: String,
) -> Result<Vec<MemoryFile>, String> {
    let memory_dir = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory");

    let read_dir = std::fs::read_dir(&memory_dir).map_err(|_| {
        format!(
            "project_not_found: No memory directory found for project '{}'",
            project_id
        )
    })?;

    let mut files = Vec::new();

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let (name, description, memory_type) = match std::fs::read_to_string(&path) {
            Ok(contents) => {
                if let Some(fm) = parse_frontmatter(&contents) {
                    (fm.name, fm.description, fm.memory_type)
                } else {
                    (None, None, None)
                }
            }
            Err(_) => (None, None, None),
        };

        files.push(MemoryFile {
            filename,
            name,
            description,
            memory_type,
        });
    }

    Ok(files)
}

pub(crate) fn get_memory_file_logic(
    state: &AppState,
    project_id: String,
    filename: String,
) -> Result<MemoryFileDetail, String> {
    let file_path = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory")
        .join(&filename);

    let content = std::fs::read_to_string(&file_path).map_err(|_| {
        format!(
            "file_not_found: Memory file '{}' not found for project '{}'",
            filename, project_id
        )
    })?;

    let (name, description, memory_type) = if let Some(fm) = parse_frontmatter(&content) {
        (fm.name, fm.description, fm.memory_type)
    } else {
        (None, None, None)
    };

    Ok(MemoryFileDetail {
        filename,
        content,
        name,
        description,
        memory_type,
    })
}

pub(crate) fn update_memory_file_logic(
    state: &AppState,
    project_id: String,
    filename: String,
    content: String,
) -> Result<(), String> {
    let file_path = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory")
        .join(&filename);

    claude_config::write::atomic_write(&file_path, content.as_bytes())
        .map_err(|e| format!("write_error: Failed to write memory file: {}", e))?;

    Ok(())
}

pub(crate) fn delete_memory_file_logic(
    state: &AppState,
    project_id: String,
    filename: String,
) -> Result<(), String> {
    let file_path = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory")
        .join(&filename);

    std::fs::remove_file(&file_path).map_err(|_| {
        format!(
            "file_not_found: Memory file '{}' not found for project '{}'",
            filename, project_id
        )
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri command shims
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_memory_projects(state: State<'_, AppState>) -> Result<Vec<MemoryProject>, String> {
    Ok(list_memory_projects_logic(&state))
}

#[tauri::command]
pub fn list_memory_files(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<MemoryFile>, String> {
    list_memory_files_logic(&state, project_id)
}

#[tauri::command]
pub fn get_memory_file(
    state: State<'_, AppState>,
    project_id: String,
    filename: String,
) -> Result<MemoryFileDetail, String> {
    get_memory_file_logic(&state, project_id, filename)
}

#[tauri::command]
pub fn update_memory_file(
    state: State<'_, AppState>,
    project_id: String,
    filename: String,
    content: String,
) -> Result<(), String> {
    update_memory_file_logic(&state, project_id, filename, content)
}

#[tauri::command]
pub fn delete_memory_file(
    state: State<'_, AppState>,
    project_id: String,
    filename: String,
) -> Result<(), String> {
    delete_memory_file_logic(&state, project_id, filename)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use tempfile::tempdir;

    // 1. list_memory_projects_empty_when_none
    #[test]
    fn list_memory_projects_empty_when_none() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let result = list_memory_projects_logic(&state);
        assert!(result.is_empty(), "expected empty list when no projects dir exists");
    }

    // 2. list_memory_projects_returns_project_with_md_files
    #[test]
    fn list_memory_projects_returns_project_with_md_files() {
        let dir = tempdir().unwrap();
        // Create projects/-Users-test-proj/memory/notes.md
        let project_dir = dir.path().join("projects").join("-Users-test-proj");
        let memory_dir = project_dir.join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();
        std::fs::write(memory_dir.join("notes.md"), "# Notes\n").unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_memory_projects_logic(&state);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "-Users-test-proj");
        // Falls back to naive decoding when no jsonl session file exists
        assert_eq!(result[0].project_path, "/Users/test/proj");
        assert_eq!(result[0].file_count, 1);
    }

    // Verify the cwd-based decoding resolves ambiguous dashes
    #[test]
    fn list_memory_projects_uses_cwd_from_session_jsonl() {
        let dir = tempdir().unwrap();
        // Ambiguous id: "-Users-eric-whoishiring-insight" could decode to
        // either "/Users/eric/whoishiring/insight" or "/Users/eric/whoishiring-insight".
        let project_id = "-Users-eric-whoishiring-insight";
        let project_dir = dir.path().join("projects").join(project_id);
        let memory_dir = project_dir.join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();
        std::fs::write(memory_dir.join("notes.md"), "# Notes\n").unwrap();

        // Create a session jsonl with the real cwd
        let real_path = "/Users/eric/whoishiring-insight";
        let session_dir = project_dir.join("session-id");
        std::fs::create_dir_all(&session_dir).unwrap();
        std::fs::write(
            session_dir.join("session.jsonl"),
            format!("{{\"cwd\":\"{}\"}}\n", real_path),
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_memory_projects_logic(&state);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].project_path, real_path);
    }

    // 3. list_memory_projects_skips_projects_without_memory_dir
    #[test]
    fn list_memory_projects_skips_projects_without_memory_dir() {
        let dir = tempdir().unwrap();
        // A project dir with no memory/ subdir
        let project_dir = dir.path().join("projects").join("-Users-no-memory");
        std::fs::create_dir_all(&project_dir).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_memory_projects_logic(&state);

        assert!(result.is_empty(), "expected empty list when no memory/ subdir");
    }

    // 4. list_memory_files_returns_md_files_with_frontmatter
    #[test]
    fn list_memory_files_returns_md_files_with_frontmatter() {
        let dir = tempdir().unwrap();
        let project_id = "-Users-test-project";
        let memory_dir = dir
            .path()
            .join("projects")
            .join(project_id)
            .join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();

        let content = "---\nname: My Note\ndescription: A test note\ntype: observation\n---\n\nBody text here.\n";
        std::fs::write(memory_dir.join("my-note.md"), content).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let files = list_memory_files_logic(&state, project_id.to_string()).unwrap();

        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.filename, "my-note.md");
        assert_eq!(f.name.as_deref(), Some("My Note"));
        assert_eq!(f.description.as_deref(), Some("A test note"));
        assert_eq!(f.memory_type.as_deref(), Some("observation"));
    }

    // 5. list_memory_files_returns_error_for_missing_project
    #[test]
    fn list_memory_files_returns_error_for_missing_project() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let err = list_memory_files_logic(&state, "nonexistent".to_string()).unwrap_err();
        assert!(
            err.starts_with("project_not_found:"),
            "expected 'project_not_found:' error, got: {}",
            err
        );
    }

    // 6. get_memory_file_returns_content_and_frontmatter
    #[test]
    fn get_memory_file_returns_content_and_frontmatter() {
        let dir = tempdir().unwrap();
        let project_id = "-Users-eric-proj";
        let memory_dir = dir
            .path()
            .join("projects")
            .join(project_id)
            .join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();

        let content = "---\nname: Detail Note\ntype: fact\n---\n\nSome detail.\n";
        std::fs::write(memory_dir.join("detail.md"), content).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let detail =
            get_memory_file_logic(&state, project_id.to_string(), "detail.md".to_string())
                .unwrap();

        assert_eq!(detail.filename, "detail.md");
        assert_eq!(detail.content, content);
        assert_eq!(detail.name.as_deref(), Some("Detail Note"));
        assert_eq!(detail.memory_type.as_deref(), Some("fact"));
        assert!(detail.description.is_none());
    }

    // 7. get_memory_file_returns_error_when_missing
    #[test]
    fn get_memory_file_returns_error_when_missing() {
        let dir = tempdir().unwrap();
        let project_id = "-Users-test";
        let memory_dir = dir
            .path()
            .join("projects")
            .join(project_id)
            .join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let err =
            get_memory_file_logic(&state, project_id.to_string(), "missing.md".to_string())
                .unwrap_err();

        assert!(
            err.starts_with("file_not_found:"),
            "expected 'file_not_found:' error, got: {}",
            err
        );
    }

    // 8. update_memory_file_writes_content
    #[test]
    fn update_memory_file_writes_content() {
        let dir = tempdir().unwrap();
        let project_id = "-Users-write-test";
        let memory_dir = dir
            .path()
            .join("projects")
            .join(project_id)
            .join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        update_memory_file_logic(
            &state,
            project_id.to_string(),
            "new-file.md".to_string(),
            "# Written by test\n".to_string(),
        )
        .unwrap();

        let on_disk = std::fs::read_to_string(memory_dir.join("new-file.md")).unwrap();
        assert_eq!(on_disk, "# Written by test\n");
    }

    // 9. delete_memory_file_removes_file
    #[test]
    fn delete_memory_file_removes_file() {
        let dir = tempdir().unwrap();
        let project_id = "-Users-delete-test";
        let memory_dir = dir
            .path()
            .join("projects")
            .join(project_id)
            .join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();
        let file_path = memory_dir.join("to-delete.md");
        std::fs::write(&file_path, "delete me\n").unwrap();
        assert!(file_path.exists());

        let state = AppState::new(dir.path().to_path_buf());
        delete_memory_file_logic(
            &state,
            project_id.to_string(),
            "to-delete.md".to_string(),
        )
        .unwrap();

        assert!(!file_path.exists(), "file should be deleted");
    }

    // 10. delete_memory_file_returns_error_when_missing
    #[test]
    fn delete_memory_file_returns_error_when_missing() {
        let dir = tempdir().unwrap();
        let project_id = "-Users-delete-err";
        let memory_dir = dir
            .path()
            .join("projects")
            .join(project_id)
            .join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let err = delete_memory_file_logic(
            &state,
            project_id.to_string(),
            "nonexistent.md".to_string(),
        )
        .unwrap_err();

        assert!(
            err.starts_with("file_not_found:"),
            "expected 'file_not_found:' error, got: {}",
            err
        );
    }
}
