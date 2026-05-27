use std::sync::Arc;
use sqlx::PgPool;
use syzygy_core::ports::{UserRepository, PostRepository};

use crate::infrastructure::postgres::{
    PgUserRepository,
    PgPostRepository,
};
use crate::infrastructure::auth::JwtService;
use crate::infrastructure::ws::ConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub user_repo: Arc<dyn UserRepository>,
    pub post_repo: Arc<dyn PostRepository>,
    pub jwt: JwtService,
    pub ws_manager: Arc<ConnectionManager>,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            user_repo: Arc::new(PgUserRepository::new(db.clone())),
            post_repo: Arc::new(PgPostRepository::new(db.clone())),
            jwt: JwtService::new(),
            ws_manager: Arc::new(ConnectionManager::new()),
            db,
        }
    }
}
