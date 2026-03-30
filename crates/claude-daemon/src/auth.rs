use axum::{
    Extension,
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use claude_types::ErrorResponse;

use crate::state::AppState;

/// Axum middleware that validates a Bearer token in the `Authorization` header.
///
/// The expected token is read from `AppState` (injected via `axum::Extension`).
/// Returns 401 if the header is missing, malformed, or contains the wrong token.
pub async fn require_auth(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Response {
    let expected = &state.inner.auth_token;

    let provided = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    match provided {
        Some(token) if token == expected => next.run(request).await,
        _ => {
            let body = ErrorResponse {
                code: "UNAUTHORIZED".to_string(),
                message: "Missing or invalid Bearer token".to_string(),
                validation_errors: vec![],
            };
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(body),
            )
                .into_response()
        }
    }
}
