use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub media_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostEnriched {
    pub id: Uuid,
    pub author: super::user::User,
    pub content: String,
    pub media_url: Option<String>,
    pub like_count: i64,
    pub syzygy_count: i64,
    pub reply_count: i64,
    pub is_liked: bool,
    pub is_syzygied: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePost {
    pub content: String,
    pub media_url: Option<String>,
}
