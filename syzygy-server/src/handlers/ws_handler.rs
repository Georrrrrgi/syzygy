use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tower_cookies::Cookies;

use crate::errors::ApiError;
use crate::state::AppState;

pub async fn ws_connect(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<impl IntoResponse, ApiError> {
    let token = cookies
        .get("syzygy_token")
        .map(|c| c.value().to_string())
        .ok_or(ApiError::Unauthorized)?;

    let claims = state.jwt.decode(&token).map_err(|_| ApiError::Unauthorized)?;
    let user_id = claims.sub;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: uuid::Uuid) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_manager.subscribe(user_id).await;

    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&event) {
                if sender.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {
            // We don't process incoming messages for now
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
