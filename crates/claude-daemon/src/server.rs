use axum::{
    Extension, Router,
    middleware,
    routing::{get, put},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::{
        config::{
            get_effective_config, get_project_config, get_user_config, put_user_config,
        },
        health::health_handler,
    },
    auth::require_auth,
    state::AppState,
};

/// Build the axum `Router` for the daemon.
///
/// Layout:
/// - `GET /api/v1/health` — public, no auth required
/// - Everything under `/api/v1/` (except health) — protected by Bearer auth
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes
    let public_routes = Router::new().route("/api/v1/health", get(health_handler));

    // Protected routes
    let protected_routes = Router::new()
        .route("/api/v1/config/user", get(get_user_config))
        .route("/api/v1/config/user", put(put_user_config))
        .route("/api/v1/config/project/{project_id}", get(get_project_config))
        .route("/api/v1/config/effective/{project_id}", get(get_effective_config))
        .layer(middleware::from_fn(require_auth));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .layer(Extension(state))
}
