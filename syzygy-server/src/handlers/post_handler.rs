use axum::{extract::{Path, State}, Json};
use axum::response::IntoResponse;
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::AuthUser;
use crate::state::AppState;
use syzygy_core::domain::CreatePost;

#[derive(Deserialize)]
pub struct CreatePostRequest {
    pub content: String,
    pub media_url: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let create = CreatePost {
        content: input.content,
        media_url: input.media_url,
    };

    let post = state.post_repo.create(&create, user.user_id).await?;

    state.ws_manager.broadcast(
        crate::infrastructure::ws::WsEvent::NewPost {
            post_id: post.id,
            author_id: user.user_id,
        },
        None,
    ).await;

    Ok(Json(post))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let post = state.post_repo.find_by_id(id).await?;
    Ok(Json(post))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.post_repo.delete(id, user.user_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn like(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.post_repo.like(user.user_id, id).await?;

    state.ws_manager.broadcast(
        crate::infrastructure::ws::WsEvent::NewLike {
            post_id: id,
            user_id: user.user_id,
        },
        None,
    ).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn unlike(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.post_repo.unlike(user.user_id, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
