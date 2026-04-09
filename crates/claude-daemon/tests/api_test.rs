use std::time::Duration;
use tempfile::TempDir;

/// Start a test daemon instance.
/// Returns (temp_dir, auth_token, port, task_handle).
/// The TempDir must be kept alive for the duration of the test.
async fn start_test_daemon() -> (TempDir, String, u16, tokio::task::JoinHandle<()>) {
    let dir = TempDir::new().unwrap();
    let claude_home = dir.path().to_path_buf();

    // Write a minimal settings.json
    std::fs::write(
        claude_home.join("settings.json"),
        r#"{"language": "en-US", "permissions": {"defaultMode": "plan"}}"#,
    )
    .unwrap();

    let token = "test-token-12345".to_string();
    let port = portpicker::pick_unused_port().unwrap();

    let state = claude_daemon::state::AppState::new(claude_home, token.clone());
    state.load_user_settings().await.unwrap();

    let app = claude_daemon::server::build_router(state);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    (dir, token, port, handle)
}

// ---------------------------------------------------------------------------
// 1. Health endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_endpoint_works() {
    let (_dir, _token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/health"))
        .send()
        .await
        .expect("health request should succeed");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(body["status"], "ok");
}

// ---------------------------------------------------------------------------
// 2. Config requires auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn config_requires_auth() {
    let (_dir, _token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    // No Authorization header
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/config/user"))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 401);
}

// ---------------------------------------------------------------------------
// 3. Get user config with auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_user_config_with_auth() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/config/user"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(body["settings"]["language"], "en-US");
}

// ---------------------------------------------------------------------------
// 4. Update user config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_user_config() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();

    // Change language to zh-CN
    let payload = serde_json::json!({
        "settings": {
            "language": "zh-CN",
            "permissions": {
                "defaultMode": "plan"
            }
        }
    });

    let resp = client
        .put(format!("http://127.0.0.1:{port}/api/v1/config/user"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(body["settings"]["language"], "zh-CN");

    // Verify the file on disk was updated
    let settings_path = dir.path().join("settings.json");
    let on_disk: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&settings_path).expect("settings file should exist"),
    )
    .expect("settings file should be valid JSON");
    assert_eq!(on_disk["language"], "zh-CN");
}

// ---------------------------------------------------------------------------
// 5. Update config with invalid defaultMode returns 422
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_config_with_invalid_mode_returns_422() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();

    // "INVALID_MODE" is not a valid defaultMode value
    let payload = serde_json::json!({
        "settings": {
            "permissions": {
                "defaultMode": "INVALID_MODE"
            }
        }
    });

    let resp = client
        .put(format!("http://127.0.0.1:{port}/api/v1/config/user"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 422);
}

// ---------------------------------------------------------------------------
// 6. Update project config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_project_config() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{port}");

    // Create a temp project directory with .claude/settings.json
    let project_dir = TempDir::new().unwrap();
    let claude_dir = project_dir.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_dir.join("settings.json"),
        r#"{"language": "en-US"}"#,
    )
    .unwrap();

    // Register the project via POST /api/v1/projects
    let payload = serde_json::json!({
        "name": "test-project",
        "path": project_dir.path().to_string_lossy()
    });
    let resp = client
        .post(format!("{base_url}/api/v1/projects"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("register request should succeed");
    assert_eq!(resp.status(), 201);
    let created: serde_json::Value = resp.json().await.expect("body should be JSON");
    let project_id = created["id"].as_str().expect("id should be a string").to_string();

    // PUT /api/v1/config/project/{id} with { settings: { language: "zh-CN" } }
    let update_payload = serde_json::json!({
        "settings": {
            "language": "zh-CN"
        }
    });
    let resp = client
        .put(format!("{base_url}/api/v1/config/project/{project_id}"))
        .bearer_auth(&token)
        .json(&update_payload)
        .send()
        .await
        .expect("put project config request should succeed");
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(body["settings"]["language"], "zh-CN");

    // Read the file from disk and verify it contains the update
    let on_disk: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(claude_dir.join("settings.json"))
            .expect("settings file should exist"),
    )
    .expect("settings file should be valid JSON");
    assert_eq!(on_disk["language"], "zh-CN");
}

// ---------------------------------------------------------------------------
// 7. Project CRUD lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn project_crud() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}/api/v1/projects");

    // --- List: should be empty initially ---
    let resp = client
        .get(&base)
        .bearer_auth(&token)
        .send()
        .await
        .expect("list request should succeed");
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(list.as_array().unwrap().len(), 0);

    // --- Register: use a real path so the validation passes ---
    let project_path = std::env::temp_dir().to_string_lossy().to_string();
    let payload = serde_json::json!({
        "name": "my-project",
        "path": project_path
    });
    let resp = client
        .post(&base)
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("register request should succeed");
    assert_eq!(resp.status(), 201);
    let created: serde_json::Value = resp.json().await.expect("body should be JSON");
    let project_id = created["id"].as_str().expect("id should be a string").to_string();
    assert!(!project_id.is_empty());

    // --- List: should now contain 1 project ---
    let resp = client
        .get(&base)
        .bearer_auth(&token)
        .send()
        .await
        .expect("list request should succeed");
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], project_id);

    // --- Unregister ---
    let resp = client
        .delete(format!("{base}/{project_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("delete request should succeed");
    assert_eq!(resp.status(), 204);

    // --- List: should be empty again ---
    let resp = client
        .get(&base)
        .bearer_auth(&token)
        .send()
        .await
        .expect("list request should succeed");
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(list.as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// 8. list_plugins_empty
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_plugins_empty() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/plugins"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(body.as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// 9. list_plugins_with_data
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_plugins_with_data() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    // Create plugins/installed_plugins.json
    let plugins_dir = dir.path().join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    std::fs::write(
        plugins_dir.join("installed_plugins.json"),
        r#"{
  "version": 2,
  "plugins": {
    "test-market": [{
      "scope": "test-plugin",
      "installPath": "/tmp/fake",
      "version": "1.0.0",
      "installedAt": "2026-01-01T00:00:00Z",
      "lastUpdated": "2026-01-01T00:00:00Z"
    }]
  }
}"#,
    )
    .unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/plugins"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "test-plugin");
    assert_eq!(arr[0]["marketplace"], "test-market");
    assert_eq!(arr[0]["version"], "1.0.0");
}

// ---------------------------------------------------------------------------
// 10. list_marketplaces_empty
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_marketplaces_empty() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/marketplaces"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(body.as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// 11. list_skills_empty
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_skills_empty() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/skills"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(body.as_array().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// 12. list_skills_with_valid_skill
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_skills_with_valid_skill() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    // Create skills/test-skill/SKILL.md with valid frontmatter
    let skill_dir = dir.path().join("skills").join("test-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: test-skill\ndescription: A test skill for integration testing\n---\n# Test Skill\nThis is a test.\n",
    )
    .unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/skills"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "test-skill");
    assert_eq!(arr[0]["name"], "test-skill");
    assert_eq!(arr[0]["valid"], true);
    assert!(arr[0]["validationError"].is_null());
}

// ---------------------------------------------------------------------------
// 13. list_skills_with_invalid_skill
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_skills_with_invalid_skill() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    // Create skills/bad-skill/SKILL.md without name field
    let skill_dir = dir.path().join("skills").join("bad-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\ndescription: Missing name field\n---\n# Bad Skill\n",
    )
    .unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/skills"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("body should be JSON");
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "bad-skill");
    assert_eq!(arr[0]["valid"], false);
    assert!(!arr[0]["validationError"].is_null());
}

// ---------------------------------------------------------------------------
// 14. memory_crud
// ---------------------------------------------------------------------------

#[tokio::test]
async fn memory_crud() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    let project_id = "-test-project";
    let filename = "test.md";
    let initial_content = "---\nname: test-memory\ndescription: A test memory file\ntype: project\n---\nTest memory content here.\n";
    let updated_content = "---\nname: test-memory\ndescription: Updated description\ntype: project\n---\nUpdated memory content.\n";

    // Create projects/-test-project/memory/ with a test .md file
    let memory_dir = dir.path().join("projects").join(project_id).join("memory");
    std::fs::create_dir_all(&memory_dir).unwrap();
    std::fs::write(memory_dir.join(filename), initial_content).unwrap();

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}/api/v1/memory");

    // GET /api/v1/memory → verify project appears
    let resp = client
        .get(&base)
        .bearer_auth(&token)
        .send()
        .await
        .expect("list projects request should succeed");
    assert_eq!(resp.status(), 200);
    let projects: serde_json::Value = resp.json().await.expect("body should be JSON");
    let projects_arr = projects.as_array().unwrap();
    assert_eq!(projects_arr.len(), 1);
    assert_eq!(projects_arr[0]["id"], project_id);
    assert_eq!(projects_arr[0]["fileCount"], 1);

    // GET /api/v1/memory/-test-project → verify file listed
    let resp = client
        .get(format!("{base}/{project_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("list files request should succeed");
    assert_eq!(resp.status(), 200);
    let files: serde_json::Value = resp.json().await.expect("body should be JSON");
    let files_arr = files.as_array().unwrap();
    assert_eq!(files_arr.len(), 1);
    assert_eq!(files_arr[0]["filename"], filename);

    // GET /api/v1/memory/-test-project/test.md → verify content
    let resp = client
        .get(format!("{base}/{project_id}/{filename}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get file request should succeed");
    assert_eq!(resp.status(), 200);
    let file_detail: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(file_detail["filename"], filename);
    assert_eq!(file_detail["content"], initial_content);
    assert_eq!(file_detail["name"], "test-memory");

    // PUT /api/v1/memory/-test-project/test.md with new content → verify 200
    let update_payload = serde_json::json!({ "content": updated_content });
    let resp = client
        .put(format!("{base}/{project_id}/{filename}"))
        .bearer_auth(&token)
        .json(&update_payload)
        .send()
        .await
        .expect("put file request should succeed");
    assert_eq!(resp.status(), 200);

    // GET again → verify updated content
    let resp = client
        .get(format!("{base}/{project_id}/{filename}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("get file request should succeed");
    assert_eq!(resp.status(), 200);
    let file_detail: serde_json::Value = resp.json().await.expect("body should be JSON");
    assert_eq!(file_detail["content"], updated_content);

    // DELETE /api/v1/memory/-test-project/test.md → verify 204
    let resp = client
        .delete(format!("{base}/{project_id}/{filename}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("delete file request should succeed");
    assert_eq!(resp.status(), 204);

    // GET list → verify file gone
    let resp = client
        .get(format!("{base}/{project_id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("list files request should succeed");
    assert_eq!(resp.status(), 200);
    let files: serde_json::Value = resp.json().await.expect("body should be JSON");
    let files_arr = files.as_array().unwrap();
    assert_eq!(files_arr.len(), 0);
}

// ---------------------------------------------------------------------------
// 15. list_mcp_servers_returns_array
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_mcp_servers_returns_array() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/mcp/servers"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    // Handler returns empty array when `claude` CLI is not available — just
    // verify the body is a JSON array regardless of its length.
    let body: Vec<serde_json::Value> = resp.json().await.expect("body should be a JSON array");
    let _ = body; // may be empty or populated depending on environment
}

// ---------------------------------------------------------------------------
// 16. launch_endpoint_exists
// ---------------------------------------------------------------------------

#[tokio::test]
async fn launch_endpoint_exists() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/api/v1/launch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "projectPath": "/tmp", "env": {} }))
        .send()
        .await
        .expect("request should succeed");

    // May be 200 (claude in PATH) or 500 (claude not in PATH).
    // We only assert the endpoint exists (not 404) and auth is accepted (not 401).
    assert_ne!(resp.status(), 404u16);
    assert_ne!(resp.status(), 401u16);
}

// ---------------------------------------------------------------------------
// Skill content endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_skill_content_returns_file() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    let skills_dir = dir.path().join("skills").join("my-test-skill");
    std::fs::create_dir_all(&skills_dir).unwrap();
    std::fs::write(
        skills_dir.join("SKILL.md"),
        "---\nname: my-test-skill\ndescription: A test skill\n---\n\n# My Test Skill\n\nThis is the content.",
    )
    .unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{port}/api/v1/skills/my-test-skill/content"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "my-test-skill");
    assert!(body["content"].as_str().unwrap().contains("# My Test Skill"));
}

#[tokio::test]
async fn get_skill_content_not_found() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{port}/api/v1/skills/nonexistent-skill/content"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// CLAUDE.md endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_claudemd_includes_global() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/api/v1/claudemd"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(body.iter().any(|f| f["id"] == "global"));
    let global = body.iter().find(|f| f["id"] == "global").unwrap();
    assert_eq!(global["exists"], false);
}

#[tokio::test]
async fn claudemd_crud_lifecycle() {
    let (dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}/api/v1/claudemd");

    // GET global — should 404 since file doesn't exist yet
    let resp = client
        .get(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    // PUT global — create the file
    let resp = client
        .put(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body("{\"content\":\"# Test CLAUDE.md\\n\\nHello world.\"}")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify file exists on disk
    assert!(dir.path().join("CLAUDE.md").exists());

    // GET global — should succeed
    let resp = client
        .get(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["content"].as_str().unwrap().contains("Test CLAUDE.md"));

    // DELETE global
    let resp = client
        .delete(format!("{base}/global"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    assert!(!dir.path().join("CLAUDE.md").exists());
}

#[tokio::test]
async fn claudemd_invalid_id_returns_400() {
    let (_dir, token, port, _handle) = start_test_daemon().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "http://127.0.0.1:{port}/api/v1/claudemd/invalid-format"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}
