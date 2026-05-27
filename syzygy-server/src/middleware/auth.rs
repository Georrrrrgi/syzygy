use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
    RequestExt,
};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::state::AppState;
use crate::errors::ApiError;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, ApiError> {
    let cookies = req.extract_parts::<Cookies>().await.map_err(|_| ApiError::Unauthorized)?;

    let token = cookies
        .get("syzygy_token")
        .map(|c| c.value().to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })
        .ok_or(ApiError::Unauthorized)?;

    let state = req.extensions().get::<AppState>().ok_or(ApiError::Internal("no app state".into()))?;
    let claims = state.jwt.decode(&token).map_err(|_| ApiError::Unauthorized)?;

    req.extensions_mut().insert(AuthUser { user_id: claims.sub });

    Ok(next.run(req).await)
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<AuthUser>().cloned().ok_or(ApiError::Unauthorized)
    }
}
