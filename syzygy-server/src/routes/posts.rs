use axum::{Router, routing::{post, get, delete}};
use crate::handlers::post_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/posts", post(post_handler::create))
        .route("/posts/:id", get(post_handler::get_by_id))
        .route("/posts/:id", delete(post_handler::delete))
        .route("/posts/:id/like", post(post_handler::like))
        .route("/posts/:id/like", delete(post_handler::unlike))
}
