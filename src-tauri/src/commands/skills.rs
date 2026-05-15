use std::collections::HashMap;
use std::path::{Path, PathBuf};

use claude_types::{
    plugins::InstalledPluginsFile,
    skills::{SkillContentResponse, SkillInfo},
};
use tauri::State;

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
    // The skills/ dir itself may be a symlink (ccs's shared pool routes
    // accounts/*/skills → ~/.ccs/shared/skills). When it is, every entry
    // below it is effectively external even though the per-entry lstat
    // looks like a plain dir.
    let dir_is_symlink = std::fs::symlink_metadata(skills_dir)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);

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

        // Mark external if either the parent skills/ dir or this skill's
        // own dir is a symlink. canonicalize() dereferences to the real
        // location so the UI can show users where it actually lives.
        let entry_is_symlink = std::fs::symlink_metadata(&entry_path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        let external_target = if dir_is_symlink || entry_is_symlink {
            std::fs::canonicalize(&entry_path)
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        } else {
            None
        };

        skills.push(SkillInfo {
            id,
            name,
            description,
            source: source.to_string(),
            path: path_str,
            valid,
            validation_error,
            external_target,
        });
    }

    skills
}

/// Find the filesystem path of a skill's SKILL.md file by its ID.
fn find_skill_path(claude_home: &Path, skill_id: &str) -> Option<PathBuf> {
    // 1. Check user skills
    let user_skill = claude_home.join("skills").join(skill_id).join("SKILL.md");
    if user_skill.exists() {
        return Some(user_skill);
    }

    // 2. Check plugin skills
    let plugins_dir = claude_home.join("plugins");
    let installed = read_installed_plugins(&plugins_dir);

    for (_marketplace_id, plugins) in &installed.plugins {
        for plugin in plugins {
            let plugin_skill = std::path::PathBuf::from(&plugin.install_path)
                .join("skills")
                .join(skill_id)
                .join("SKILL.md");
            if plugin_skill.exists() {
                return Some(plugin_skill);
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Logic helpers (testable without Tauri State)
// ---------------------------------------------------------------------------

pub(crate) fn list_skills_logic(claude_home: &Path) -> Vec<SkillInfo> {
    let mut result = Vec::new();

    // 1. User skills: {claude_home}/skills/<subdirectory>/SKILL.md
    let user_skills_dir = claude_home.join("skills");
    result.extend(scan_skills_dir(&user_skills_dir, "user"));

    // 2. Plugin skills: for each installed plugin, check {install_path}/skills/
    let plugins_dir = claude_home.join("plugins");
    let installed = read_installed_plugins(&plugins_dir);

    // Map keys look like "plugin-name@marketplace-id" (e.g. "superpowers@claude-plugins-official").
    // `plugin.scope` is the install scope ("user" | "project"), not the plugin name.
    for (plugin_key, plugins) in &installed.plugins {
        for plugin in plugins {
            let source = format!("plugin:{}", plugin_key);
            let plugin_skills_dir =
                std::path::PathBuf::from(&plugin.install_path).join("skills");
            result.extend(scan_skills_dir(&plugin_skills_dir, &source));
        }
    }

    result
}

pub(crate) fn get_skill_content_logic(
    claude_home: &Path,
    id: String,
) -> Result<SkillContentResponse, String> {
    let skill_path = find_skill_path(claude_home, &id)
        .ok_or_else(|| format!("not_found: Skill '{}' not found", id))?;

    let content = std::fs::read_to_string(&skill_path)
        .map_err(|e| format!("read_error: Failed to read skill file: {}", e))?;

    Ok(SkillContentResponse { id, content })
}

/// Delete a user-level skill (`<claude_home>/skills/<id>/`). Plugin-owned
/// skills are not removable through this path — the caller must uninstall
/// the owning plugin instead. If the entry is a symlink (e.g. a
/// ccs-shared pool that survived ccs-migration), only the symlink is
/// removed; the target is left intact.
pub(crate) fn delete_user_skill_logic(claude_home: &Path, id: &str) -> Result<(), String> {
    // Reject path traversal / nested ids — only a plain directory name.
    if id.is_empty() || id.contains('/') || id.contains('\\') || id == "." || id == ".." {
        return Err(format!("invalid_id: '{}' is not a valid skill id", id));
    }

    let skill_dir = claude_home.join("skills").join(id);
    let meta = match std::fs::symlink_metadata(&skill_dir) {
        Ok(m) => m,
        Err(_) => {
            return Err(format!(
                "not_found: User skill '{}' not found at {}",
                id,
                skill_dir.display()
            ));
        }
    };

    if meta.file_type().is_symlink() {
        std::fs::remove_file(&skill_dir)
            .map_err(|e| format!("delete_failed: {}: {}", skill_dir.display(), e))
    } else if meta.is_dir() {
        std::fs::remove_dir_all(&skill_dir)
            .map_err(|e| format!("delete_failed: {}: {}", skill_dir.display(), e))
    } else {
        Err(format!(
            "unexpected_kind: {} is neither a directory nor a symlink",
            skill_dir.display()
        ))
    }
}

// ---------------------------------------------------------------------------
// Tauri command shims
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn list_skills(state: State<'_, AppState>) -> Result<Vec<SkillInfo>, String> {
    let claude_home = state.current_dir().await;
    Ok(list_skills_logic(&claude_home))
}

#[tauri::command]
pub async fn get_skill_content(
    state: State<'_, AppState>,
    id: String,
) -> Result<SkillContentResponse, String> {
    let claude_home = state.current_dir().await;
    get_skill_content_logic(&claude_home, id)
}

#[tauri::command]
pub async fn delete_user_skill(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let claude_home = state.current_dir().await;
    delete_user_skill_logic(&claude_home, &id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_skills_returns_empty_when_no_skills_dir() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let result = list_skills_logic(&state.inner.claude_home);
        assert!(result.is_empty(), "expected empty list when no skills dir exists");
    }

    #[tokio::test]
    async fn list_skills_finds_user_skill_with_frontmatter() {
        let dir = tempdir().unwrap();
        // Create skills/<skill-name>/SKILL.md
        let skill_dir = dir.path().join("skills").join("my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: My Skill\ndescription: Does something useful\n---\n\n# Body\n",
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        let result = list_skills_logic(&state.inner.claude_home);

        assert_eq!(result.len(), 1);
        let skill = &result[0];
        assert_eq!(skill.id, "my-skill");
        assert_eq!(skill.name, "My Skill");
        assert_eq!(skill.description.as_deref(), Some("Does something useful"));
        assert_eq!(skill.source, "user");
        assert!(skill.valid);
        assert!(skill.validation_error.is_none());
        assert!(
            skill.external_target.is_none(),
            "plain dir should not be marked external"
        );
    }

    /// When the whole `skills/` dir is a symlink (the ccs-shared-pool case),
    /// every entry under it must be flagged with the real target path so the
    /// UI can surface that these skills are not account-local.
    #[cfg(unix)]
    #[tokio::test]
    async fn list_skills_flags_external_when_skills_dir_is_symlink() {
        let dir = tempdir().unwrap();
        // Real skills pool elsewhere
        let real_pool = dir.path().join("real-pool");
        let real_skill_dir = real_pool.join("shared-skill");
        std::fs::create_dir_all(&real_skill_dir).unwrap();
        std::fs::write(
            real_skill_dir.join("SKILL.md"),
            "---\nname: Shared Skill\ndescription: lives in shared pool\n---\n",
        )
        .unwrap();

        // claude_home/skills → real-pool
        let claude_home = dir.path().join("home");
        std::fs::create_dir_all(&claude_home).unwrap();
        std::os::unix::fs::symlink(&real_pool, claude_home.join("skills")).unwrap();

        let result = list_skills_logic(&claude_home);

        assert_eq!(result.len(), 1);
        let skill = &result[0];
        assert_eq!(skill.id, "shared-skill");
        let target = skill
            .external_target
            .as_deref()
            .expect("external_target should be set when skills/ is a symlink");
        assert!(
            target.contains("real-pool"),
            "external_target should point at real pool, got: {}",
            target
        );
    }

    #[tokio::test]
    async fn delete_user_skill_removes_real_dir() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("skills").join("my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: My Skill\ndescription: x\n---\n",
        )
        .unwrap();

        delete_user_skill_logic(dir.path(), "my-skill").unwrap();
        assert!(!skill_dir.exists(), "skill dir should be removed");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn delete_user_skill_removes_symlink_but_not_target() {
        let dir = tempdir().unwrap();
        // Real skill lives outside the account
        let real_dir = dir.path().join("real-pool").join("shared");
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(real_dir.join("SKILL.md"), "---\nname: Shared\n---\n").unwrap();

        // skills/shared → real-pool/shared
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        std::os::unix::fs::symlink(&real_dir, skills_dir.join("shared")).unwrap();

        delete_user_skill_logic(dir.path(), "shared").unwrap();
        assert!(
            !skills_dir.join("shared").exists(),
            "symlink itself should be gone"
        );
        assert!(
            real_dir.join("SKILL.md").exists(),
            "symlink target must NOT be touched"
        );
    }

    #[tokio::test]
    async fn delete_user_skill_returns_not_found_for_unknown_id() {
        let dir = tempdir().unwrap();
        let err = delete_user_skill_logic(dir.path(), "missing-skill").unwrap_err();
        assert!(
            err.starts_with("not_found:"),
            "expected not_found error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn delete_user_skill_rejects_path_traversal() {
        let dir = tempdir().unwrap();
        for bad in ["..", ".", "../escape", "foo/bar", ""] {
            let err = delete_user_skill_logic(dir.path(), bad).unwrap_err();
            assert!(
                err.starts_with("invalid_id:") || err.starts_with("not_found:"),
                "expected invalid_id for '{}', got: {}",
                bad,
                err
            );
        }
    }

    #[tokio::test]
    async fn get_skill_content_returns_not_found_for_unknown_id() {
        let dir = tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());

        let err = get_skill_content_logic(&state.inner.claude_home, "nonexistent-skill".to_string())
            .unwrap_err();

        assert!(
            err.starts_with("not_found:"),
            "expected error starting with 'not_found:', got: {}",
            err
        );
    }
}
