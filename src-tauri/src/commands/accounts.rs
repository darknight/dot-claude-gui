use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Account directory management.
//
// Each account is a directory under <app_config>/accounts/<name>/ that
// Claude Code uses as its home (via the CLAUDE_CONFIG_DIR env var). This
// module owns disk operations only; per-account metadata (createdAt etc.) is
// reconciled on the frontend against AppConfig.accounts in config.json.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskAccount {
    pub name: String,
    /// Directory mtime as Unix seconds (UTC). Frontend converts to ISO 8601
    /// and uses it as a fallback `createdAt` when config.json has no metadata.
    pub created_at_unix: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountStatus {
    pub logged_in: bool,
    pub email: Option<String>,
}

fn home_dir() -> Result<PathBuf, String> {
    dirs_next::home_dir().ok_or_else(|| "cannot determine home directory".to_string())
}

fn accounts_root_under(app_dir: &Path) -> PathBuf {
    app_dir.join("accounts")
}

fn default_app_dir() -> Result<PathBuf, String> {
    home_dir().map(|h| h.join(".dot-claude-gui"))
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("invalid_name: empty".into());
    }
    if name.len() > 32 {
        return Err("invalid_name: too long (max 32)".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err("invalid_name: only [a-z0-9_-] allowed".into());
    }
    Ok(())
}

fn mtime_unix(path: &Path) -> u64 {
    let mtime = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or_else(|_| SystemTime::now());
    mtime
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn list_accounts_in(app_dir: &Path) -> Result<Vec<DiskAccount>, String> {
    let root = accounts_root_under(app_dir);
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&root)
        .map_err(|e| format!("failed to read accounts dir: {e}"))?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if validate_name(&name).is_err() {
            continue; // ignore unrelated subdirs
        }
        out.push(DiskAccount {
            created_at_unix: mtime_unix(&entry.path()),
            name,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn create_account_in(app_dir: &Path, name: &str) -> Result<DiskAccount, String> {
    validate_name(name)?;
    let root = accounts_root_under(app_dir);
    std::fs::create_dir_all(&root)
        .map_err(|e| format!("failed to create accounts dir: {e}"))?;
    let dir = root.join(name);
    if dir.exists() {
        return Err(format!("account_exists: {name}"));
    }
    std::fs::create_dir(&dir)
        .map_err(|e| format!("failed to create account dir: {e}"))?;
    Ok(DiskAccount {
        name: name.to_string(),
        created_at_unix: mtime_unix(&dir),
    })
}

fn delete_account_in(app_dir: &Path, name: &str) -> Result<(), String> {
    validate_name(name)?;
    let root = accounts_root_under(app_dir);
    let dir = root.join(name);
    if !dir.exists() {
        return Err(format!("account_missing: {name}"));
    }
    // Defence-in-depth: ensure we resolve back inside the accounts root.
    let canonical = dir
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize: {e}"))?;
    let root_canonical = root
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize root: {e}"))?;
    if !canonical.starts_with(&root_canonical) {
        return Err("invalid_name: path traversal".into());
    }
    std::fs::remove_dir_all(&canonical)
        .map_err(|e| format!("failed to remove account dir: {e}"))
}

/// Read login status from `<account_dir>/.claude.json`. Claude Code writes
/// `oauthAccount.emailAddress` (and other org metadata) on successful login;
/// presence of that field is our authoritative "logged in" signal. Missing
/// file or missing field => not logged in (no error).
fn read_account_status_in(app_dir: &Path, name: &str) -> Result<AccountStatus, String> {
    validate_name(name)?;
    let path = accounts_root_under(app_dir).join(name).join(".claude.json");
    if !path.exists() {
        return Ok(AccountStatus::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("read .claude.json: {e}"))?;
    let v: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("parse .claude.json: {e}"))?;
    let email = v
        .pointer("/oauthAccount/emailAddress")
        .and_then(|x| x.as_str())
        .map(str::to_string);
    Ok(AccountStatus {
        logged_in: email.is_some(),
        email,
    })
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn list_accounts() -> Result<Vec<DiskAccount>, String> {
    let app_dir = default_app_dir()?;
    list_accounts_in(&app_dir)
}

#[tauri::command]
pub fn create_account(name: String) -> Result<DiskAccount, String> {
    let app_dir = default_app_dir()?;
    create_account_in(&app_dir, &name)
}

#[tauri::command]
pub fn delete_account(name: String) -> Result<(), String> {
    let app_dir = default_app_dir()?;
    delete_account_in(&app_dir, &name)
}

#[tauri::command]
pub fn get_account_status(name: String) -> Result<AccountStatus, String> {
    let app_dir = default_app_dir()?;
    read_account_status_in(&app_dir, &name)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_name_accepts_valid() {
        assert!(validate_name("work").is_ok());
        assert!(validate_name("me").is_ok());
        assert!(validate_name("work-2").is_ok());
        assert!(validate_name("a_b_c").is_ok());
        assert!(validate_name("a1b2").is_ok());
    }

    #[test]
    fn validate_name_rejects_invalid() {
        assert!(validate_name("").is_err());
        assert!(validate_name("Work").is_err()); // uppercase
        assert!(validate_name("a/b").is_err()); // slash
        assert!(validate_name("..").is_err()); // dots
        assert!(validate_name(&"a".repeat(33)).is_err()); // > 32
        assert!(validate_name("中文").is_err());
        assert!(validate_name("with space").is_err());
    }

    #[test]
    fn create_account_creates_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let acct = create_account_in(tmp.path(), "work").unwrap();
        assert_eq!(acct.name, "work");
        assert!(tmp.path().join("accounts/work").is_dir());
        assert!(acct.created_at_unix > 0);
    }

    #[test]
    fn create_account_rejects_duplicate() {
        let tmp = tempfile::tempdir().unwrap();
        create_account_in(tmp.path(), "work").unwrap();
        let err = create_account_in(tmp.path(), "work").unwrap_err();
        assert!(err.starts_with("account_exists:"));
    }

    #[test]
    fn delete_account_removes_dir() {
        let tmp = tempfile::tempdir().unwrap();
        create_account_in(tmp.path(), "work").unwrap();
        delete_account_in(tmp.path(), "work").unwrap();
        assert!(!tmp.path().join("accounts/work").exists());
    }

    #[test]
    fn delete_account_rejects_path_traversal() {
        let tmp = tempfile::tempdir().unwrap();
        // Path traversal blocked at validate_name (slashes/dots forbidden).
        let err = delete_account_in(tmp.path(), "../foo").unwrap_err();
        assert!(err.starts_with("invalid_name:"));
    }

    #[test]
    fn delete_account_missing_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = delete_account_in(tmp.path(), "ghost").unwrap_err();
        assert!(err.starts_with("account_missing:"));
    }

    #[test]
    fn list_accounts_includes_orphan_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        create_account_in(tmp.path(), "work").unwrap();
        // Orphan: directory created without going through create_account.
        std::fs::create_dir_all(tmp.path().join("accounts/orphan")).unwrap();
        let listed = list_accounts_in(tmp.path()).unwrap();
        let names: Vec<&str> = listed.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["orphan", "work"]);
    }

    #[test]
    fn list_accounts_returns_empty_when_root_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let listed = list_accounts_in(tmp.path()).unwrap();
        assert!(listed.is_empty());
    }

    #[test]
    fn list_accounts_skips_invalid_names() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("accounts/work")).unwrap();
        std::fs::create_dir_all(tmp.path().join("accounts/.cache")).unwrap();
        let listed = list_accounts_in(tmp.path()).unwrap();
        let names: Vec<&str> = listed.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, vec!["work"]);
    }

    fn write_claude_json(app_dir: &Path, name: &str, body: &str) {
        let dir = app_dir.join("accounts").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".claude.json"), body).unwrap();
    }

    #[test]
    fn read_account_status_returns_default_when_no_claude_json() {
        let tmp = tempfile::tempdir().unwrap();
        create_account_in(tmp.path(), "work").unwrap();
        let status = read_account_status_in(tmp.path(), "work").unwrap();
        assert!(!status.logged_in);
        assert_eq!(status.email, None);
    }

    #[test]
    fn read_account_status_extracts_email_when_oauth_present() {
        let tmp = tempfile::tempdir().unwrap();
        create_account_in(tmp.path(), "work").unwrap();
        write_claude_json(
            tmp.path(),
            "work",
            r#"{"oauthAccount":{"emailAddress":"a@b.com","organizationName":"Acme"}}"#,
        );
        let status = read_account_status_in(tmp.path(), "work").unwrap();
        assert!(status.logged_in);
        assert_eq!(status.email.as_deref(), Some("a@b.com"));
    }

    #[test]
    fn read_account_status_returns_not_logged_in_when_oauth_missing() {
        let tmp = tempfile::tempdir().unwrap();
        create_account_in(tmp.path(), "work").unwrap();
        write_claude_json(tmp.path(), "work", r#"{"userID":"x","numStartups":1}"#);
        let status = read_account_status_in(tmp.path(), "work").unwrap();
        assert!(!status.logged_in);
        assert_eq!(status.email, None);
    }

    #[test]
    fn read_account_status_handles_malformed_json() {
        let tmp = tempfile::tempdir().unwrap();
        create_account_in(tmp.path(), "work").unwrap();
        write_claude_json(tmp.path(), "work", "{not json");
        let err = read_account_status_in(tmp.path(), "work").unwrap_err();
        assert!(err.starts_with("parse .claude.json:"));
    }
}
