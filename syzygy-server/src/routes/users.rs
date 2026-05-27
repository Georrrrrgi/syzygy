use axum::{Router, routing::{get, put, post, delete}};
use crate::handlers::user_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users/:id", get(user_handler::get_by_id))
        .route("/users/:id", put(user_handler::update))
        .route("/users/:id/follow", post(user_handler::follow))
        .route("/users/:id/follow", delete(user_handler::unfollow))
        .route("/users/:id/followers", get(user_handler::get_followers))
        .route("/users/:id/following", get(user_handler::get_following))
        .route("/users/:id/posts", get(user_handler::get_posts))
        .route("/users/search", get(user_handler::search))
}
