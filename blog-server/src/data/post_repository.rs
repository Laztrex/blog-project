//! Реализация репозитория постов на PostgreSQL.

use crate::domain::error::DomainError;
use crate::domain::post::Post;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: &Post) -> Result<Post, DomainError>;
    async fn find_by_id(&self, id: i64) -> Result<Post, DomainError>;
    async fn update(&self, id: i64, title: &str, content: &str) -> Result<Post, DomainError>;
    async fn delete(&self, id: i64) -> Result<(), DomainError>;
    async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Post>, i64), DomainError>;
}

pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, post: &Post) -> Result<Post, DomainError> {
        let row = sqlx::query(
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.author_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(Post {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            author_id: row.try_get("author_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<Post, DomainError> {
        let row = sqlx::query(
            "SELECT id, title, content, author_id, created_at, updated_at FROM posts WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(row) => Ok(Post {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                content: row.try_get("content")?,
                author_id: row.try_get("author_id")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            }),
            None => Err(DomainError::PostNotFound),
        }
    }

    async fn update(&self, id: i64, title: &str, content: &str) -> Result<Post, DomainError> {
        let row = sqlx::query(
            "UPDATE posts SET title = $1, content = $2 WHERE id = $3 RETURNING id, title, content, author_id, created_at, updated_at"
        )
        .bind(title)
        .bind(content)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(Post {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            author_id: row.try_get("author_id")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    async fn delete(&self, id: i64) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Post>, i64), DomainError> {
        let total: i64 = sqlx::query("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await?
            .try_get(0)?;
        let rows = sqlx::query(
            "SELECT id, title, content, author_id, created_at, updated_at FROM posts ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        let posts = rows
            .iter()
            .map(|row| Post {
                id: row.try_get("id").unwrap(),
                title: row.try_get("title").unwrap(),
                content: row.try_get("content").unwrap(),
                author_id: row.try_get("author_id").unwrap(),
                created_at: row.try_get("created_at").unwrap(),
                updated_at: row.try_get("updated_at").unwrap(),
            })
            .collect();
        Ok((posts, total))
    }
}
