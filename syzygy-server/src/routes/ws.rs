use axum::{Router, routing::get};
use crate::handlers::ws_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ws", get(ws_handler::ws_connect))
}
