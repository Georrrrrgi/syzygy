use axum::{extract::{Query, State}, Json};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::errors::ApiError;
use crate::middleware::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct FeedQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_feed(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let posts = state.post_repo.get_feed(user.user_id, limit, offset).await?;
    Ok(Json(posts))
}

pub async fn get_explore(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let posts = state.post_repo.get_explore(user.user_id, limit, offset).await?;
    Ok(Json(posts))
}
