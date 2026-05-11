// src-tauri/src/commands/gui_projects.rs
//
// Project binding CRUD. Reads/writes ~/.dot-claude-gui/config.json directly.
// All operations are atomic at the file level (see app_config::write_config).

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::app_config::{
    read_config, write_config, AppConfig, LaunchConfig, ProjectBinding, DEFAULT_ACCOUNT_NAME,
};

fn config_path() -> Result<PathBuf, String> {
    let dir = dirs_next::home_dir()
        .ok_or("cannot determine home directory")?
        .join(".dot-claude-gui");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir config dir: {e}"))?;
    Ok(dir.join("config.json"))
}

fn mutate<F>(f: F) -> Result<AppConfig, String>
where F: FnOnce(&mut AppConfig) -> Result<(), String>
{
    let path = config_path()?;
    let mut cfg = read_config(&path)?;
    f(&mut cfg)?;
    write_config(&path, &cfg)?;
    Ok(cfg)
}

// ── List ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntry {
    pub path: String,
    /// `None` => unbound; `Some(name)` => bound to that account.
    pub account: Option<String>,
    pub launch: LaunchConfig,
    /// True when `path` doesn't exist on disk.
    pub stale: bool,
}

#[tauri::command]
pub fn gui_list_projects() -> Result<Vec<ProjectEntry>, String> {
    let cfg = read_config(&config_path()?)?;
    let entries = cfg.known_projects.iter().map(|path| {
        let stale = !std::path::Path::new(path).exists();
        let (account, launch) = match cfg.projects.get(path) {
            Some(b) => (Some(b.account.clone()), b.launch.clone()),
            None    => (None, LaunchConfig::default()),
        };
        ProjectEntry { path: path.clone(), account, launch, stale }
    }).collect();
    Ok(entries)
}

// ── Add (registers a path; no binding yet) ──────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProjectRequest { pub path: String }

#[tauri::command]
pub fn add_project(req: AddProjectRequest) -> Result<ProjectEntry, String> {
    let p = std::path::PathBuf::from(&req.path);
    if !p.exists() {
        return Err(format!("invalid_path: {}", req.path));
    }
    let path = p.canonicalize()
        .map_err(|e| format!("canonicalize path: {e}"))?
        .to_string_lossy()
        .to_string();

    mutate(|cfg| {
        if !cfg.known_projects.contains(&path) {
            cfg.known_projects.push(path.clone());
        }
        Ok(())
    })?;

    Ok(ProjectEntry { path, account: None, launch: LaunchConfig::default(), stale: false })
}

// ── Bind / Unbind ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindProjectRequest { pub path: String, pub account: String }

#[tauri::command]
pub fn bind_project(req: BindProjectRequest) -> Result<(), String> {
    mutate(|cfg| {
        let known_account =
            req.account == DEFAULT_ACCOUNT_NAME ||
            cfg.accounts.iter().any(|a| a.name == req.account);
        if !known_account {
            return Err(format!("unknown_account: {}", req.account));
        }
        if !cfg.known_projects.contains(&req.path) {
            cfg.known_projects.push(req.path.clone());
        }
        cfg.projects.entry(req.path.clone())
            .or_insert_with(ProjectBinding::default)
            .account = req.account.clone();
        Ok(())
    })?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnbindProjectRequest { pub path: String }

#[tauri::command]
pub fn unbind_project(req: UnbindProjectRequest) -> Result<(), String> {
    mutate(|cfg| { cfg.projects.remove(&req.path); Ok(()) })?;
    Ok(())
}

// ── Remove (from list entirely) ─────────────────────────────────────────

#[tauri::command]
pub fn remove_project(req: UnbindProjectRequest) -> Result<(), String> {
    mutate(|cfg| {
        cfg.projects.remove(&req.path);
        cfg.known_projects.retain(|p| p != &req.path);
        Ok(())
    })?;
    Ok(())
}

// ── Update launch ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLaunchRequest { pub path: String, pub launch: LaunchConfig }

#[tauri::command]
pub fn update_project_launch(req: UpdateLaunchRequest) -> Result<(), String> {
    mutate(|cfg| {
        let entry = cfg.projects.entry(req.path.clone())
            .or_insert_with(ProjectBinding::default);
        entry.launch = req.launch.clone();
        Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::{write_config, AppConfig, Account};

    fn isolated() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        // Override HOME for this thread so `config_path()` resolves into the tempdir.
        std::env::set_var("HOME", dir.path());
        dir
    }

    #[test]
    #[serial_test::serial]
    fn add_then_list_includes_path_as_unbound() {
        let _g = isolated();
        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        add_project(AddProjectRequest { path: real.clone() }).unwrap();
        let list = gui_list_projects().unwrap();
        assert!(list.iter().any(|p| p.path == real && p.account.is_none()));
    }

    #[test]
    #[serial_test::serial]
    fn bind_then_list_shows_binding() {
        let _g = isolated();
        // Seed with a known account so bind_project accepts it.
        let cfg_path = config_path().unwrap();
        let mut cfg = AppConfig::default();
        cfg.accounts.push(Account {
            name: "work".into(), display_name: "work".into(),
            is_native: false, created_at: "x".into(),
        });
        write_config(&cfg_path, &cfg).unwrap();

        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        bind_project(BindProjectRequest { path: real.clone(), account: "work".into() }).unwrap();
        let list = gui_list_projects().unwrap();
        let entry = list.iter().find(|p| p.path == real).expect("bound entry present");
        assert_eq!(entry.account.as_deref(), Some("work"));
    }

    #[test]
    #[serial_test::serial]
    fn bind_rejects_unknown_account() {
        let _g = isolated();
        let res = bind_project(BindProjectRequest {
            path: "/some/path".into(),
            account: "ghost".into(),
        });
        assert!(res.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn bind_accepts_default_without_explicit_seed() {
        let _g = isolated();
        // Native ~/.claude/ check uses real HOME which we've overridden; create
        // the dir so the "default" account exists conceptually.
        std::fs::create_dir_all(std::env::var("HOME").map(std::path::PathBuf::from).unwrap().join(".claude")).unwrap();
        let res = bind_project(BindProjectRequest {
            path: "/some/path".into(),
            account: "default".into(),
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn remove_drops_from_known_and_projects() {
        let _g = isolated();
        let real = std::env::current_dir().unwrap().to_string_lossy().to_string();
        add_project(AddProjectRequest { path: real.clone() }).unwrap();
        remove_project(UnbindProjectRequest { path: real.clone() }).unwrap();
        let list = gui_list_projects().unwrap();
        assert!(list.iter().all(|p| p.path != real));
    }

    #[test]
    #[serial_test::serial]
    fn list_marks_stale_paths() {
        let _g = isolated();
        let cfg_path = config_path().unwrap();
        let mut cfg = AppConfig::default();
        cfg.known_projects.push("/definitely/does/not/exist/12345".into());
        write_config(&cfg_path, &cfg).unwrap();

        let list = gui_list_projects().unwrap();
        let entry = list.iter().find(|p| p.path == "/definitely/does/not/exist/12345").unwrap();
        assert!(entry.stale);
    }
}
