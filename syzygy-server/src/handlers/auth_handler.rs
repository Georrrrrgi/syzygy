use axum::{extract::State, Json};
use axum::response::IntoResponse;
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};

use crate::errors::ApiError;
use crate::middleware::AuthUser;
use crate::state::AppState;
use syzygy_core::domain::CreateUser;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(input): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let password_hash = bcrypt::hash(&input.password, bcrypt::DEFAULT_COST)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let create_user = CreateUser {
        username: input.username,
        display_name: input.display_name,
        bio: input.bio,
        password: input.password,
    };

    let user = state.user_repo.create(&create_user, &password_hash).await?;

    let token = state.jwt.encode(user.id).map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut cookie = Cookie::new("syzygy_token", token.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_max_age(time::Duration::days(7));
    cookies.add(cookie);

    Ok(Json(serde_json::json!({
        "user": user,
        "token": token,
    })))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(input): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let (user, password_hash) = state.user_repo
        .find_by_username_with_password(&input.username)
        .await?;

    let valid = bcrypt::verify(&input.password, &password_hash)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !valid {
        return Err(ApiError::Domain(syzygy_core::domain::DomainError::InvalidCredentials));
    }

    let token = state.jwt.encode(user.id).map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut cookie = Cookie::new("syzygy_token", token.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_max_age(time::Duration::days(7));
    cookies.add(cookie);

    Ok(Json(serde_json::json!({
        "user": user,
        "token": token,
    })))
}

pub async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let user = state.user_repo.find_by_id(user.user_id).await?;
    Ok(Json(user))
}
