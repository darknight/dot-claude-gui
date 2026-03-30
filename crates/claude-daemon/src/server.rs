use axum::{
    Extension, Router,
    middleware,
    routing::{delete, get, post, put},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::{
        config::{
            get_effective_config, get_project_config, get_user_config, put_user_config,
        },
        health::health_handler,
        projects::{delete_project, list_projects, register_project},
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
        .route("/api/v1/config/project/{project_id}", get(get_project_config))
        .route("/api/v1/config/effective/{project_id}", get(get_effective_config))
        .route("/api/v1/projects", get(list_projects))
        .route("/api/v1/projects", post(register_project))
        .route("/api/v1/projects/{id}", delete(delete_project))
        .layer(middleware::from_fn(require_auth));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .layer(Extension(state))
}
