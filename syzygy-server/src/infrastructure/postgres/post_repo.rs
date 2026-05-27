use async_trait::async_trait;
use sqlx::{PgPool, Row};
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

    fn row_to_post(row: &sqlx::postgres::PgRow) -> Post {
        Post {
            id: row.get("id"),
            author_id: row.get("author_id"),
            content: row.get("content"),
            media_url: row.get("media_url"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_user(row: &sqlx::postgres::PgRow) -> User {
        User {
            id: row.get("id"),
            username: row.get("username"),
            display_name: row.get("display_name"),
            bio: row.get("bio"),
            avatar_url: row.get("avatar_url"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_like(row: &sqlx::postgres::PgRow) -> Like {
        Like {
            user_id: row.get("user_id"),
            post_id: row.get("post_id"),
            created_at: row.get("created_at"),
        }
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

        let row = sqlx::query(
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
        })?;

        Ok(Self::row_to_post(&row))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<PostEnriched, DomainError> {
        let row = sqlx::query(
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

        let author_id: Uuid = row.get("author_id");
        let author_row = sqlx::query(
            "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE id = $1",
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding post author: {e}");
            DomainError::UserNotFound
        })?;

        let counts = sqlx::query(
            r#"SELECT
                (SELECT COUNT(*) FROM likes WHERE post_id = $1) as like_count,
                (SELECT COUNT(*) FROM syzygies WHERE post_id = $1) as syzygy_count"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting post counts: {e}");
            DomainError::PostNotFound
        })?;

        Ok(PostEnriched {
            id: row.get("id"),
            author: Self::row_to_user(&author_row),
            content: row.get("content"),
            media_url: row.get("media_url"),
            like_count: counts.get("like_count"),
            syzygy_count: counts.get("syzygy_count"),
            reply_count: 0,
            is_liked: false,
            is_syzygied: false,
            created_at: row.get("created_at"),
        })
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<(), DomainError> {
        let row = sqlx::query("SELECT author_id FROM posts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error finding post for delete: {e}");
                DomainError::PostNotFound
            })?
            .ok_or(DomainError::PostNotFound)?;

        let author_id: Uuid = row.get("author_id");
        if author_id != user_id {
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
        let rows = sqlx::query(
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

        self.enrich_posts(&rows, user_id).await
    }

    async fn get_explore(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError> {
        let rows = sqlx::query(
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

        self.enrich_posts(&rows, user_id).await
    }

    async fn get_by_user(&self, author_id: Uuid, viewer_id: Uuid, limit: i64, offset: i64) -> Result<Vec<PostEnriched>, DomainError> {
        let rows = sqlx::query(
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

        self.enrich_posts(&rows, viewer_id).await
    }

    async fn like(&self, user_id: Uuid, post_id: Uuid) -> Result<(), DomainError> {
        let now = chrono::Utc::now();
        sqlx::query("INSERT INTO likes (user_id, post_id, created_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
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
        let rows = sqlx::query(
            "SELECT user_id, post_id, created_at FROM likes WHERE post_id = $1 ORDER BY created_at DESC",
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting likes: {e}");
            DomainError::PostNotFound
        })?;

        Ok(rows.iter().map(Self::row_to_like).collect())
    }
}

impl PgPostRepository {
    async fn enrich_posts(
        &self,
        rows: &[sqlx::postgres::PgRow],
        viewer_id: Uuid,
    ) -> Result<Vec<PostEnriched>, DomainError> {
        let mut enriched = Vec::with_capacity(rows.len());

        for row in rows {
            let author_id: Uuid = row.get("author_id");
            let post_id: Uuid = row.get("id");

            let author_row = sqlx::query(
                "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE id = $1",
            )
            .bind(author_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error enriching posts: {e}");
                DomainError::UserNotFound
            })?;

            let counts = sqlx::query(
                r#"SELECT
                    (SELECT COUNT(*) FROM likes WHERE post_id = $1) as like_count,
                    (SELECT COUNT(*) FROM syzygies WHERE post_id = $1) as syzygy_count"#,
            )
            .bind(post_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error getting counts: {e}");
                DomainError::PostNotFound
            })?;

            let like_count: i64 = sqlx::query("SELECT COUNT(*) as cnt FROM likes WHERE user_id = $1 AND post_id = $2")
                .bind(viewer_id)
                .bind(post_id)
                .fetch_one(&self.pool)
                .await
                .map(|r| r.get::<i64, _>("cnt"))
                .unwrap_or(0);

            enriched.push(PostEnriched {
                id: post_id,
                author: Self::row_to_user(&author_row),
                content: row.get("content"),
                media_url: row.get("media_url"),
                like_count: counts.get("like_count"),
                syzygy_count: counts.get("syzygy_count"),
                reply_count: 0,
                is_liked: like_count > 0,
                is_syzygied: false,
                created_at: row.get("created_at"),
            });
        }

        Ok(enriched)
    }
}
