use axum::{
    Extension, Router,
    middleware,
    routing::get,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{api::health::health_handler, auth::require_auth, state::AppState};

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

    // Protected routes — placeholder, more routes added in Tasks 7-8
    let protected_routes = Router::new()
        .layer(middleware::from_fn(require_auth));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .layer(Extension(state))
}
