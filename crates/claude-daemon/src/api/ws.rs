use axum::{
    Extension,
    extract::{
        Query,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use claude_types::api::ErrorResponse;
use claude_types::events::{WsClientMessage, WsEvent};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tracing::{debug, info};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/v1/ws
// ---------------------------------------------------------------------------

/// WebSocket upgrade handler. Authenticates via `?token=` query param.
pub async fn ws_handler(
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
    Extension(state): Extension<AppState>,
) -> Response {
    // Authenticate via query-param token (WS cannot send custom headers easily).
    match &query.token {
        Some(token) if token == &state.inner.auth_token => {}
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    code: "UNAUTHORIZED".to_string(),
                    message: "Invalid or missing token".to_string(),
                    validation_errors: vec![],
                }),
            )
                .into_response();
        }
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Send Connected event immediately after handshake.
    let connected_event = WsEvent::Connected {
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    if let Ok(json) = serde_json::to_string(&connected_event) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    // Subscribe to the broadcast channel.
    let mut rx = state.inner.ws_tx.subscribe();

    info!("WebSocket client connected");

    loop {
        tokio::select! {
            // Forward broadcast events to the client.
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        match serde_json::to_string(&event) {
                            Ok(json) => {
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                debug!("Failed to serialize WsEvent: {}", e);
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        debug!("WebSocket broadcast lagged by {} messages", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            // Receive messages from the client.
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsClientMessage>(&text) {
                            Ok(WsClientMessage::Subscribe { topics }) => {
                                debug!("Client subscribed to topics: {:?}", topics);
                            }
                            Ok(WsClientMessage::Unsubscribe { topics }) => {
                                debug!("Client unsubscribed from topics: {:?}", topics);
                            }
                            Err(e) => {
                                debug!("Failed to parse client message: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ignore binary / ping / pong frames.
                    }
                    Some(Err(e)) => {
                        debug!("WebSocket receive error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket client disconnected");
}
