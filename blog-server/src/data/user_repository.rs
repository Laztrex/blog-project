//! Реализация репозитория пользователей на PostgreSQL.

use crate::domain::error::DomainError;
use crate::domain::user::User;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<User, DomainError>;
    async fn find_by_id(&self, id: i64) -> Result<User, DomainError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, DomainError> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        Ok(User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
        })
    }

    async fn find_by_username(&self, username: &str) -> Result<User, DomainError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Ok(User {
                id: row.try_get("id")?,
                username: row.try_get("username")?,
                email: row.try_get("email")?,
                password_hash: row.try_get("password_hash")?,
                created_at: row.try_get("created_at")?,
            }),
            None => Err(DomainError::UserNotFound),
        }
    }

    async fn find_by_id(&self, id: i64) -> Result<User, DomainError> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Ok(User {
                id: row.try_get("id")?,
                username: row.try_get("username")?,
                email: row.try_get("email")?,
                password_hash: row.try_get("password_hash")?,
                created_at: row.try_get("created_at")?,
            }),
            None => Err(DomainError::UserNotFound),
        }
    }
}
