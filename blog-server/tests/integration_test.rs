#[cfg(test)]
mod tests {
    use blog_server::data::{
        post_repository::{PostRepository, PostgresPostRepository},
        user_repository::{PostgresUserRepository, UserRepository},
    };
    use blog_server::domain::post::Post;
    use sqlx::PgPool;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// get_test_pool создаёт пул подключений к тестовой БД и применяет миграции.
    async fn get_test_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost/blog_test".to_string());
        let pool = sqlx::PgPool::connect(&database_url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(pool)
    }

    fn unique_name(base: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
        format!("{}_{}", base, timestamp)
    }

    #[tokio::test]
    async fn test_user_crud() -> Result<(), Box<dyn std::error::Error>> {
        let pool = get_test_pool().await?;
        let repo = PostgresUserRepository::new(pool);
        let username = unique_name("testuser");
        let user = repo
            .create(&username, &format!("{}@ex.com", username), "hash")
            .await?;
        assert_eq!(user.username, username);
        let found = repo.find_by_username(&username).await?;
        assert_eq!(found.id, user.id);
        let by_id = repo.find_by_id(user.id).await?;
        assert_eq!(by_id.email, format!("{}@ex.com", username));
        Ok(())
    }

    #[tokio::test]
    async fn test_post_crud() -> Result<(), Box<dyn std::error::Error>> {
        let pool = get_test_pool().await?;
        let user_repo = PostgresUserRepository::new(pool.clone());
        let post_repo = PostgresPostRepository::new(pool);
        let username = unique_name("author");
        let user = user_repo
            .create(&username, &format!("{}@ex.com", username), "hash")
            .await?;
        let post = Post::new("Title".to_string(), "Content".to_string(), user.id);
        let created = post_repo.create(&post).await?;
        assert_eq!(created.title, "Title");
        let fetched = post_repo.find_by_id(created.id).await?;
        assert_eq!(fetched.content, "Content");
        let updated = post_repo.update(created.id, "New", "New content").await?;
        assert_eq!(updated.title, "New");
        post_repo.delete(created.id).await?;
        let result = post_repo.find_by_id(created.id).await;
        assert!(result.is_err());
        Ok(())
    }
}
