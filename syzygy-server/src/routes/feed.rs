use axum::{Router, routing::get};
use crate::handlers::feed_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/feed", get(feed_handler::get_feed))
        .route("/explore", get(feed_handler::get_explore))
}
