use axum::{
    Extension, Router,
    middleware,
    routing::{delete, get, post, put},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::{
        config::{
            get_effective_config, get_project_config, get_user_config, put_project_config,
            put_user_config,
        },
        health::health_handler,
        launcher::launch_claude,
        memory::{
            delete_memory_file, get_memory_file, list_memory_files, list_memory_projects,
            put_memory_file,
        },
        plugins::{
            add_marketplace, browse_marketplace_plugins, install_plugin, list_marketplaces,
            list_plugins, remove_marketplace, toggle_plugin, uninstall_plugin,
        },
        projects::{delete_project, list_projects, register_project},
        mcp::{add_mcp_server, list_mcp_servers, remove_mcp_server},
        skills::list_skills,
        ws::ws_handler,
    },
    auth::require_auth,
    state::AppState,
};

/// Build the axum `Router` for the daemon.
///
/// Layout:
/// - `GET /api/v1/health`   — public, no auth required
/// - `GET /api/v1/ws`       — public, authenticates via `?token=` query param
/// - Everything else        — protected by Bearer auth middleware
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth middleware).
    let public_routes = Router::new()
        .route("/api/v1/health", get(health_handler))
        .route("/api/v1/ws", get(ws_handler));

    // Protected routes (Bearer token middleware).
    let protected_routes = Router::new()
        .route("/api/v1/config/user", get(get_user_config))
        .route("/api/v1/config/user", put(put_user_config))
        .route(
            "/api/v1/config/project/{project_id}",
            get(get_project_config).put(put_project_config),
        )
        .route("/api/v1/config/effective/{project_id}", get(get_effective_config))
        .route("/api/v1/projects", get(list_projects))
        .route("/api/v1/projects", post(register_project))
        .route("/api/v1/projects/{id}", delete(delete_project))
        // Plugin routes
        .route("/api/v1/plugins", get(list_plugins))
        .route("/api/v1/plugins/install", post(install_plugin))
        .route("/api/v1/plugins/{id}/toggle", post(toggle_plugin))
        .route("/api/v1/plugins/{id}/uninstall", post(uninstall_plugin))
        // Marketplace routes
        .route("/api/v1/marketplaces", get(list_marketplaces).post(add_marketplace))
        .route("/api/v1/marketplaces/{id}/plugins", get(browse_marketplace_plugins))
        .route("/api/v1/marketplaces/{id}", delete(remove_marketplace))
        // MCP routes
        .route("/api/v1/mcp/servers", get(list_mcp_servers).post(add_mcp_server))
        .route("/api/v1/mcp/servers/{name}", delete(remove_mcp_server))
        // Launcher route
        .route("/api/v1/launch", post(launch_claude))
        // Skills routes
        .route("/api/v1/skills", get(list_skills))
        // Memory routes
        .route("/api/v1/memory", get(list_memory_projects))
        .route("/api/v1/memory/{project_id}", get(list_memory_files))
        .route(
            "/api/v1/memory/{project_id}/{filename}",
            get(get_memory_file).put(put_memory_file).delete(delete_memory_file),
        )
        .layer(middleware::from_fn(require_auth));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .layer(Extension(state))
}
