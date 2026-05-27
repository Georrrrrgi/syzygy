use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use syzygy_core::domain::{CreateUser, DomainError, UpdateUser, User, UserProfile};
use syzygy_core::ports::UserRepository;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, input: &CreateUser, password_hash: &str) -> Result<User, DomainError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let existing = sqlx::query("SELECT COUNT(*) as cnt FROM users WHERE username = $1")
            .bind(&input.username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error checking username: {e}");
                DomainError::UsernameAlreadyExists
            })?;

        let count: i64 = existing.get("cnt");
        if count > 0 {
            return Err(DomainError::UsernameAlreadyExists);
        }

        let display = if input.display_name.is_empty() {
            &input.username
        } else {
            &input.display_name
        };

        let row = sqlx::query(
            r#"INSERT INTO users (id, username, display_name, bio, password_hash, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, username, display_name, bio, avatar_url, created_at"#,
        )
        .bind(id)
        .bind(&input.username)
        .bind(display)
        .bind(&input.bio)
        .bind(password_hash)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error creating user: {e}");
            DomainError::UsernameAlreadyExists
        })?;

        Ok(Self::row_to_user(&row))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        let row = sqlx::query(
            "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding user: {e}");
            DomainError::UserNotFound
        })?
        .ok_or(DomainError::UserNotFound)?;

        Ok(Self::row_to_user(&row))
    }

    async fn find_by_username(&self, username: &str) -> Result<User, DomainError> {
        let row = sqlx::query(
            "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding user by username: {e}");
            DomainError::UserNotFound
        })?
        .ok_or(DomainError::UserNotFound)?;

        Ok(Self::row_to_user(&row))
    }

    async fn find_by_username_with_password(&self, username: &str) -> Result<(User, String), DomainError> {
        let row = sqlx::query(
            "SELECT id, username, display_name, bio, avatar_url, password_hash, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error: {e}");
            DomainError::UserNotFound
        })?
        .ok_or(DomainError::UserNotFound)?;

        let user = User {
            id: row.get("id"),
            username: row.get("username"),
            display_name: row.get("display_name"),
            bio: row.get("bio"),
            avatar_url: row.get("avatar_url"),
            created_at: row.get("created_at"),
        };
        let password_hash: String = row.get("password_hash");
        Ok((user, password_hash))
    }

    async fn update(&self, id: Uuid, input: &UpdateUser) -> Result<User, DomainError> {
        let user = self.find_by_id(id).await?;

        let display_name = input.display_name.clone().unwrap_or(user.display_name);
        let bio = input.bio.clone().unwrap_or(user.bio);
        let avatar_url = input.avatar_url.clone().or(user.avatar_url);

        let row = sqlx::query(
            r#"UPDATE users
               SET display_name = $1, bio = $2, avatar_url = $3
               WHERE id = $4
               RETURNING id, username, display_name, bio, avatar_url, created_at"#,
        )
        .bind(display_name)
        .bind(bio)
        .bind(avatar_url)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error updating user: {e}");
            DomainError::UserNotFound
        })?;

        Ok(Self::row_to_user(&row))
    }

    async fn get_profile(&self, id: Uuid) -> Result<UserProfile, DomainError> {
        let user = self.find_by_id(id).await?;

        let row = sqlx::query(
            r#"SELECT
                (SELECT COUNT(*) FROM follows WHERE followee_id = $1) as follower_count,
                (SELECT COUNT(*) FROM follows WHERE follower_id = $1) as following_count,
                (SELECT COUNT(*) FROM posts WHERE author_id = $1) as post_count"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error getting profile counts: {e}");
            DomainError::UserNotFound
        })?;

        Ok(UserProfile {
            user,
            follower_count: row.get("follower_count"),
            following_count: row.get("following_count"),
            post_count: row.get("post_count"),
        })
    }

    async fn search(&self, query_str: &str, limit: i64, offset: i64) -> Result<Vec<User>, DomainError> {
        let pattern = format!("%{}%", query_str);
        let rows = sqlx::query(
            r#"SELECT u.id, u.username, u.display_name, u.bio, u.avatar_url, u.created_at
               FROM users u
               WHERE u.username ILIKE $1 OR u.display_name ILIKE $1
               ORDER BY u.created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error searching users: {e}");
            DomainError::UserNotFound
        })?;

        Ok(rows.iter().map(Self::row_to_user).collect())
    }
}
