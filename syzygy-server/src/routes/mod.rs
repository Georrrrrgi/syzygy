pub mod auth;
pub mod posts;
pub mod users;
pub mod feed;
pub mod ws;

use axum::{Router, middleware};
use tower_http::cors::{CorsLayer, Any};
use tower_cookies::CookieManagerLayer;
use crate::state::AppState;
use crate::middleware::auth_middleware;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_routes = Router::new()
        .merge(auth::routes())
        .merge(ws::routes());

    let protected_routes = Router::new()
        .merge(posts::routes())
        .merge(users::routes())
        .merge(feed::routes())
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .nest("/api", public_routes)
        .nest("/api", protected_routes)
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .with_state(state)
}
