use crate::domain::{CreatePost, DomainError, Like, Post, PostEnriched};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, input: &CreatePost, author_id: Uuid) -> Result<Post, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<PostEnriched, DomainError>;
    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), DomainError>;
    async fn get_feed(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError>;
    async fn get_explore(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError>;
    async fn get_by_user(&self, author_id: Uuid, viewer_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError>;
    async fn like(&self, user_id: Uuid, post_id: Uuid) -> Result<(), DomainError>;
    async fn unlike(&self, user_id: Uuid, post_id: Uuid) -> Result<(), DomainError>;
    async fn get_likes(&self, post_id: Uuid) -> Result<Vec<Like>, DomainError>;
}
