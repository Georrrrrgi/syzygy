use crate::domain::{CreateUser, DomainError, UpdateUser, User, UserProfile};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, input: &CreateUser, password_hash: &str) -> Result<User, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<User, DomainError>;
    async fn find_by_username_with_password(&self, username: &str) -> Result<(User, String), DomainError>;
    async fn update(&self, id: Uuid, input: &UpdateUser) -> Result<User, DomainError>;
    async fn get_profile(&self, id: Uuid) -> Result<UserProfile, DomainError>;
    async fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<User>, DomainError>;
}
