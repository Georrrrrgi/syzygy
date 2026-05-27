use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use syzygy_core::domain::{CreatePost, DomainError, Like, Post, PostEnriched, User};
use syzygy_core::ports::PostRepository;

pub struct PgPostRepository {
    pool: PgPool,
}

impl PgPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PgPostRepository {
    async fn create(&self, input: &CreatePost, author_id: Uuid) -> Result<Post, DomainError> {
        let content = input.content.trim().to_string();
        if content.is_empty() {
            return Err(DomainError::EmptyContent);
        }
        if content.len() > 777 {
            return Err(DomainError::ContentTooLong);
        }

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query_as::<_, Post>(
            r#"INSERT INTO posts (id, author_id, content, media_url, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, author_id, content, media_url, created_at, updated_at"#,
        )
        .bind(id)
        .bind(author_id)
        .bind(&content)
        .bind(&input.media_url)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error creating post: {e}");
            DomainError::ContentTooLong
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<PostEnriched, DomainError> {
        let row = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, author_id, content, media_url, created_at, updated_at FROM posts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding post: {e}");
            DomainError::PostNotFound
        })?
        .ok_or(DomainError::PostNotFound)?;

        let author = sqlx::query_as::<_, User>(
            "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE id = $1",
        )
        .bind(row.1)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding post author: {e}");
            DomainError::UserNotFound
        })?;

        let counts = sqlx::query_as::<_, (i64, i64, i64)>(
            r#"SELECT
                (SELECT COUNT(*) FROM likes WHERE post_id = $1) as like_count,
                (SELECT COUNT(*) FROM syzygies WHERE post_id = $1) as syzygy_count,
                0 as reply_count"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting post counts: {e}");
            DomainError::PostNotFound
        })?;

        Ok(PostEnriched {
            id: row.0,
            author,
            content: row.2,
            media_url: row.3,
            like_count: counts.0,
            syzygy_count: counts.1,
            reply_count: counts.2,
            is_liked: false,
            is_syzygied: false,
            created_at: row.4,
        })
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), DomainError> {
        let post = sqlx::query_as::<_, (Uuid,)>(
            "SELECT author_id FROM posts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding post for delete: {e}");
            DomainError::PostNotFound
        })?
        .ok_or(DomainError::PostNotFound)?;

        if post.0 != user_id {
            return Err(DomainError::Unauthorized);
        }

        sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error deleting post: {e}");
                DomainError::PostNotFound
            })?;

        Ok(())
    }

    async fn get_feed(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT p.id, p.author_id, p.content, p.media_url, p.created_at, p.updated_at
               FROM posts p
               INNER JOIN follows f ON f.followee_id = p.author_id
               WHERE f.follower_id = $1
               ORDER BY p.created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting feed: {e}");
            DomainError::PostNotFound
        })?;

        self.enrich_posts(rows, user_id).await
    }

    async fn get_explore(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT p.id, p.author_id, p.content, p.media_url, p.created_at, p.updated_at
               FROM posts p
               ORDER BY p.created_at DESC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting explore: {e}");
            DomainError::PostNotFound
        })?;

        self.enrich_posts(rows, user_id).await
    }

    async fn get_by_user(&self, author_id: Uuid, viewer_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"SELECT p.id, p.author_id, p.content, p.media_url, p.created_at, p.updated_at
               FROM posts p
               WHERE p.author_id = $1
               ORDER BY p.created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(author_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting user posts: {e}");
            DomainError::PostNotFound
        })?;

        self.enrich_posts(rows, viewer_id).await
    }

    async fn like(&self, user_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        let now = chrono::Utc::now();
        sqlx::query(
            "INSERT INTO likes (user_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(post_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error liking post: {e}");
            DomainError::PostNotFound
        })?;
        Ok(())
    }

    async fn unlike(&self, user_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM likes WHERE user_id = $1 AND post_id = $2")
            .bind(user_id)
            .bind(post_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error unliking post: {e}");
                DomainError::PostNotFound
            })?;
        Ok(())
    }

    async fn get_likes(&self, post_id: Uuid) -> Result<Vec<Like>, DomainError> {
        sqlx::query_as::<_, Like>(
            "SELECT user_id, post_id, created_at FROM likes WHERE post_id = $1 ORDER BY created_at DESC",
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting likes: {e}");
            DomainError::PostNotFound
        })
    }
}

impl PgPostRepository {
    async fn enrich_posts(
        &self,
        rows: Vec<(Uuid, Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
        viewer_id: Uuid,
    ) -> Result<Vec<PostEnriched>, DomainError> {
        let mut enriched = Vec::with_capacity(rows.len());

        for row in rows {
            let author = sqlx::query_as::<_, User>(
                "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE id = $1",
            )
            .bind(row.1)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error enriching posts: {e}");
                DomainError::UserNotFound
            })?;

            let counts = sqlx::query_as::<_, (i64, i64, i64)>(
                r#"SELECT
                    (SELECT COUNT(*) FROM likes WHERE post_id = $1) as like_count,
                    (SELECT COUNT(*) FROM syzygies WHERE post_id = $1) as syzygy_count,
                    0 as reply_count"#,
            )
            .bind(row.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error getting counts: {e}");
                DomainError::PostNotFound
            })?;

            let is_liked = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM likes WHERE user_id = $1 AND post_id = $2",
            )
            .bind(viewer_id)
            .bind(row.0)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0) > 0;

            enriched.push(PostEnriched {
                id: row.0,
                author,
                content: row.2,
                media_url: row.3,
                like_count: counts.0,
                syzygy_count: counts.1,
                reply_count: counts.2,
                is_liked,
                is_syzygied: false,
                created_at: row.4,
            });
        }

        Ok(enriched)
    }
}
