use axum::{
    extract::{Path, Query, State},
    Json,
};
use axum::response::IntoResponse;
use serde::Deserialize;
use uuid::Uuid;

use crate::errors::ApiError;
use crate::middleware::AuthUser;
use crate::state::AppState;
use syzygy_core::domain::UpdateUser;

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let profile = state.user_repo.get_profile(id).await?;
    Ok(Json(profile))
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if id != user.user_id {
        return Err(ApiError::Domain(syzygy_core::domain::DomainError::Unauthorized));
    }

    let update = UpdateUser {
        display_name: input.display_name,
        bio: input.bio,
        avatar_url: input.avatar_url,
    };

    let user = state.user_repo.update(id, &update).await?;
    Ok(Json(user))
}

pub async fn follow(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    if id == user.user_id {
        return Err(ApiError::Domain(syzygy_core::domain::DomainError::CannotFollowSelf));
    }

    sqlx::query(
        "INSERT INTO follows (follower_id, followee_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(user.user_id)
    .bind(id)
    .bind(chrono::Utc::now())
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    state.ws_manager.broadcast(
        crate::infrastructure::ws::WsEvent::NewFollow {
            follower_id: user.user_id,
            followee_id: id,
        },
        None,
    ).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn unfollow(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    sqlx::query("DELETE FROM follows WHERE follower_id = $1 AND followee_id = $2")
        .bind(user.user_id)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_followers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);

    let users = sqlx::query_as::<_, syzygy_core::domain::User>(
        r#"SELECT u.id, u.username, u.display_name, u.bio, u.avatar_url, u.created_at
           FROM users u
           INNER JOIN follows f ON f.follower_id = u.id
           WHERE f.followee_id = $1
           ORDER BY f.created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(users))
}

pub async fn get_following(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);

    let users = sqlx::query_as::<_, syzygy_core::domain::User>(
        r#"SELECT u.id, u.username, u.display_name, u.bio, u.avatar_url, u.created_at
           FROM users u
           INNER JOIN follows f ON f.followee_id = u.id
           WHERE f.follower_id = $1
           ORDER BY f.created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(users))
}

pub async fn get_posts(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = pagination.limit.unwrap_or(20).min(100);
    let offset = pagination.offset.unwrap_or(0);

    let posts = state.post_repo.get_by_user(id, user.user_id, limit, offset).await?;
    Ok(Json(posts))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let users = state.user_repo.search(&query.q, limit, offset).await?;
    Ok(Json(users))
}
