use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
};
use claude_types::{
    api::ErrorResponse,
    memory::{MemoryFile, MemoryFileDetail, MemoryProject, UpdateMemoryRequest},
};

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
/// Directory names encode paths by replacing `/` with `-`.
/// e.g. `-Users-eric-yao-workspace-darknight` → `/Users/eric.yao/workspace/darknight`
fn decode_project_path(project_id: &str) -> String {
    project_id.replace('-', "/")
}

// ---------------------------------------------------------------------------
// GET /api/v1/memory
// ---------------------------------------------------------------------------

/// List all projects that have a `memory/` subdirectory with at least one `.md` file.
pub async fn list_memory_projects(
    Extension(state): Extension<AppState>,
) -> Json<Vec<MemoryProject>> {
    let projects_dir = state.inner.claude_home.join("projects");

    let read_dir = match std::fs::read_dir(&projects_dir) {
        Ok(rd) => rd,
        Err(_) => return Json(vec![]),
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
            project_path: decode_project_path(&id),
            id,
            file_count,
        });
    }

    Json(result)
}

// ---------------------------------------------------------------------------
// GET /api/v1/memory/:project_id
// ---------------------------------------------------------------------------

/// List all memory files for a given project.
pub async fn list_memory_files(
    Extension(state): Extension<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<MemoryFile>>, (StatusCode, Json<ErrorResponse>)> {
    let memory_dir = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory");

    let read_dir = std::fs::read_dir(&memory_dir).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "PROJECT_NOT_FOUND".to_string(),
                message: format!("No memory directory found for project '{}'", project_id),
                validation_errors: vec![],
            }),
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

    Ok(Json(files))
}

// ---------------------------------------------------------------------------
// GET /api/v1/memory/:project_id/:filename
// ---------------------------------------------------------------------------

/// Read the full content of a single memory file.
pub async fn get_memory_file(
    Extension(state): Extension<AppState>,
    Path((project_id, filename)): Path<(String, String)>,
) -> Result<Json<MemoryFileDetail>, (StatusCode, Json<ErrorResponse>)> {
    let file_path = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory")
        .join(&filename);

    let content = std::fs::read_to_string(&file_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: format!("Memory file '{}' not found for project '{}'", filename, project_id),
                validation_errors: vec![],
            }),
        )
    })?;

    let (name, description, memory_type) = if let Some(fm) = parse_frontmatter(&content) {
        (fm.name, fm.description, fm.memory_type)
    } else {
        (None, None, None)
    };

    Ok(Json(MemoryFileDetail {
        filename,
        content,
        name,
        description,
        memory_type,
    }))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/memory/:project_id/:filename
// ---------------------------------------------------------------------------

/// Update the content of a memory file atomically.
pub async fn put_memory_file(
    Extension(state): Extension<AppState>,
    Path((project_id, filename)): Path<(String, String)>,
    Json(body): Json<UpdateMemoryRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let file_path = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory")
        .join(&filename);

    claude_config::write::atomic_write(&file_path, body.content.as_bytes()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "WRITE_ERROR".to_string(),
                message: format!("Failed to write memory file: {}", e),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/memory/:project_id/:filename
// ---------------------------------------------------------------------------

/// Delete a memory file.
pub async fn delete_memory_file(
    Extension(state): Extension<AppState>,
    Path((project_id, filename)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let file_path = state
        .inner
        .claude_home
        .join("projects")
        .join(&project_id)
        .join("memory")
        .join(&filename);

    std::fs::remove_file(&file_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: format!("Memory file '{}' not found for project '{}'", filename, project_id),
                validation_errors: vec![],
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
