use async_trait::async_trait;
use sqlx::PgPool;
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
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, input: &CreateUser, password_hash: &str) -> Result<User, DomainError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = $1",
        )
        .bind(&input.username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error checking username: {e}");
            DomainError::UsernameAlreadyExists
        })?;

        if existing > 0 {
            return Err(DomainError::UsernameAlreadyExists);
        }

        let display = if input.display_name.is_empty() {
            &input.username
        } else {
            &input.display_name
        };

        sqlx::query_as::<_, User>(
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
        })
    }

    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError> {
        sqlx::query_as::<_, User>(
            "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding user: {e}");
            DomainError::UserNotFound
        })?
        .ok_or(DomainError::UserNotFound)
    }

    async fn find_by_username(&self, username: &str) -> Result<User, DomainError> {
        sqlx::query_as::<_, User>(
            "SELECT id, username, display_name, bio, avatar_url, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding user by username: {e}");
            DomainError::UserNotFound
        })?
        .ok_or(DomainError::UserNotFound)
    }

    async fn find_by_username_with_password(&self, username: &str) -> Result<(User, String), DomainError> {
        let row = sqlx::query_as::<_, (Uuid, String, String, String, Option<String>, String, chrono::DateTime<chrono::Utc>)>(
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
            id: row.0,
            username: row.1,
            display_name: row.2,
            bio: row.3,
            avatar_url: row.4,
            created_at: row.6,
        };
        Ok((user, row.5))
    }

    async fn update(&self, id: Uuid, input: &UpdateUser) -> Result<User, DomainError> {
        let user = self.find_by_id(id).await?;

        let display_name = input.display_name.clone().unwrap_or(user.display_name);
        let bio = input.bio.clone().unwrap_or(user.bio);
        let avatar_url = input.avatar_url.clone().or(user.avatar_url);

        sqlx::query_as::<_, User>(
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
        })
    }

    async fn get_profile(&self, id: Uuid) -> Result<UserProfile, DomainError> {
        let user = self.find_by_id(id).await?;

        let counts = sqlx::query_as::<_, (i64, i64, i64)>(
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
            follower_count: counts.0,
            following_count: counts.1,
            post_count: counts.2,
        })
    }

    async fn search(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<User>, DomainError> {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, User>(
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
        })
    }
}
