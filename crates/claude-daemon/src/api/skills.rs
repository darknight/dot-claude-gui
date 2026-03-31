use std::collections::HashMap;
use std::path::Path;

use axum::{Extension, Json};
use claude_types::{
    plugins::InstalledPluginsFile,
    skills::SkillInfo,
};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// SKILL.md frontmatter parsing
// ---------------------------------------------------------------------------

struct FrontmatterResult {
    name: Option<String>,
    description: Option<String>,
}

/// Parse YAML frontmatter from a SKILL.md file's contents.
///
/// A valid frontmatter block:
/// - Line 0 must be `---`
/// - Subsequent lines are scanned for `name:` and `description:` until the
///   closing `---` is found.
fn parse_frontmatter(contents: &str) -> Option<FrontmatterResult> {
    let mut lines = contents.lines();

    // First line must be exactly `---`.
    if lines.next()?.trim() != "---" {
        return None;
    }

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
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
        }
    }

    if !closed {
        return None;
    }

    Some(FrontmatterResult { name, description })
}

/// Validate a parsed frontmatter result and return `(valid, validation_error)`.
fn validate_frontmatter(
    result: Option<FrontmatterResult>,
) -> (Option<FrontmatterResult>, bool, Option<String>) {
    match result {
        None => (
            None,
            false,
            Some("missing or malformed frontmatter block".to_string()),
        ),
        Some(fm) => {
            let missing_name = fm.name.is_none();
            let missing_desc = fm.description.is_none();

            let (valid, error) = match (missing_name, missing_desc) {
                (false, false) => (true, None),
                (true, false) => (
                    false,
                    Some("missing 'name' field in frontmatter".to_string()),
                ),
                (false, true) => (
                    false,
                    Some("missing 'description' field in frontmatter".to_string()),
                ),
                (true, true) => (
                    false,
                    Some(
                        "missing 'name' and 'description' fields in frontmatter".to_string(),
                    ),
                ),
            };

            (Some(fm), valid, error)
        }
    }
}

// ---------------------------------------------------------------------------
// Skill scanning helpers
// ---------------------------------------------------------------------------

/// Read and parse installed_plugins.json, returning an empty default on failure.
fn read_installed_plugins(plugins_dir: &Path) -> InstalledPluginsFile {
    let path = plugins_dir.join("installed_plugins.json");
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return InstalledPluginsFile { version: 1, plugins: HashMap::new() },
    };
    serde_json::from_str(&contents).unwrap_or_else(|_| InstalledPluginsFile {
        version: 1,
        plugins: HashMap::new(),
    })
}

/// Scan a single `skills/` directory and return a list of `SkillInfo` entries.
///
/// `source` is the string that will be placed in `SkillInfo::source`
/// (e.g. `"user"` or `"plugin:myplugin@marketplace"`).
fn scan_skills_dir(skills_dir: &Path, source: &str) -> Vec<SkillInfo> {
    let read_dir = match std::fs::read_dir(skills_dir) {
        Ok(rd) => rd,
        Err(_) => return vec![],
    };

    let mut skills = Vec::new();

    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let id = match entry_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let skill_md_path = entry_path.join("SKILL.md");
        if !skill_md_path.exists() {
            continue;
        }

        let path_str = skill_md_path.to_string_lossy().into_owned();

        let (name, description, valid, validation_error) =
            match std::fs::read_to_string(&skill_md_path) {
                Err(_) => (
                    id.clone(),
                    None,
                    false,
                    Some("could not read SKILL.md".to_string()),
                ),
                Ok(contents) => {
                    let parsed = parse_frontmatter(&contents);
                    let (fm, valid, error) = validate_frontmatter(parsed);
                    let name = fm
                        .as_ref()
                        .and_then(|f| f.name.clone())
                        .unwrap_or_else(|| id.clone());
                    let description = fm.and_then(|f| f.description);
                    (name, description, valid, error)
                }
            };

        skills.push(SkillInfo {
            id,
            name,
            description,
            source: source.to_string(),
            path: path_str,
            valid,
            validation_error,
        });
    }

    skills
}

// ---------------------------------------------------------------------------
// GET /api/v1/skills
// ---------------------------------------------------------------------------

pub async fn list_skills(
    Extension(state): Extension<AppState>,
) -> Json<Vec<SkillInfo>> {
    let claude_home = &state.inner.claude_home;
    let mut result = Vec::new();

    // 1. User skills: {claude_home}/skills/<subdirectory>/SKILL.md
    let user_skills_dir = claude_home.join("skills");
    result.extend(scan_skills_dir(&user_skills_dir, "user"));

    // 2. Plugin skills: for each installed plugin, check {install_path}/skills/
    let plugins_dir = claude_home.join("plugins");
    let installed = read_installed_plugins(&plugins_dir);

    for (marketplace_id, plugins) in &installed.plugins {
        for plugin in plugins {
            let plugin_id = format!("{}@{}", plugin.scope, marketplace_id);
            let source = format!("plugin:{}", plugin_id);
            let plugin_skills_dir =
                std::path::PathBuf::from(&plugin.install_path).join("skills");
            result.extend(scan_skills_dir(&plugin_skills_dir, &source));
        }
    }

    Json(result)
}
