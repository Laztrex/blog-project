use crate::domain::error::DomainError;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, DomainError> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;
    Ok(pool)
}
