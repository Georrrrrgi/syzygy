use axum::{Router, routing::{post, get}};
use crate::handlers::auth_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(auth_handler::register))
        .route("/auth/login", post(auth_handler::login))
        .route("/auth/me", get(auth_handler::me))
}
