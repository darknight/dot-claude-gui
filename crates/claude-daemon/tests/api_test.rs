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
