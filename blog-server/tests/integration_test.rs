#[cfg(test)]
mod tests {
    use blog_server::data::{
        post_repository::{PostRepository, PostgresPostRepository},
        user_repository::{PostgresUserRepository, UserRepository},
    };
    use blog_server::domain::post::Post;
    use sqlx::PgPool;

    /// prepare_database подготавливает тестовую базу данных: применяет миграции, если таблицы не существуют.
    /// Эта функция вызывается один раз в каждом тесте.
    async fn prepare_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::migrate!().run(pool).await?;
        Ok(())
    }

    // sqlx::test автоматически создаёт транзакцию и откатывает её после теста,
    // данные не остаются в БД.
    #[sqlx::test]
    async fn test_user_crud(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        prepare_database(&pool).await?;
        let repo = PostgresUserRepository::new(pool);
        let user = repo.create("testuser", "test@ex.com", "hash").await?;
        assert_eq!(user.username, "testuser");
        let found = repo.find_by_username("testuser").await?;
        assert_eq!(found.id, user.id);
        let by_id = repo.find_by_id(user.id).await?;
        assert_eq!(by_id.email, "test@ex.com");
        Ok(())
    }

    #[sqlx::test]
    async fn test_post_crud(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
        prepare_database(&pool).await?;
        let user_repo = PostgresUserRepository::new(pool.clone());
        let post_repo = PostgresPostRepository::new(pool);
        let user = user_repo.create("author", "author@ex.com", "hash").await?;
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
