use std::sync::Arc;
use tokio::select;
use tracing::{error, info};

use blog_server::application::{auth_service::AuthService, blog_service::BlogService};
use blog_server::data::{
    post_repository::PostgresPostRepository, user_repository::PostgresUserRepository,
};
use blog_server::infrastructure::{
    config::AppConfig, database::create_pool, jwt::JwtService, logging::init_logging,
};
use blog_server::presentation;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_logging();

    let config = AppConfig::from_env();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = create_pool(&database_url, config.db_max_connections).await?;
    sqlx::migrate!().run(&pool).await?;
    info!("Database migrations applied");

    let jwt_service = Arc::new(JwtService::new(&jwt_secret, config.jwt_expiry_hours));

    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()))
        as Arc<dyn blog_server::data::user_repository::UserRepository>;
    let post_repo = Arc::new(PostgresPostRepository::new(pool.clone()))
        as Arc<dyn blog_server::data::post_repository::PostRepository>;

    let auth_service = Arc::new(AuthService::new(user_repo.clone(), jwt_service.clone()));
    let blog_service = Arc::new(BlogService::new(post_repo.clone(), user_repo.clone()));

    let http_server = presentation::http_handlers::run_http_server(
        auth_service.clone(),
        blog_service.clone(),
        jwt_service.clone(),
        config.http_port,
    )?;

    let grpc_server = match presentation::grpc_service::run_grpc_server(
        auth_service,
        blog_service,
        jwt_service,
        config.grpc_port,
    ) {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to start gRPC server: {}", e);
            return Err(anyhow::anyhow!("{}", e));
        }
    };

    info!(
        "Starting servers: HTTP on 0.0.0.0:{}, gRPC on 0.0.0.0:{}",
        config.http_port, config.grpc_port
    );
    select! {
        res = http_server => {
            if let Err(e) = res {
                error!("HTTP server error: {}", e);
            }
        }
        res = grpc_server => {
            if let Err(e) = res {
                error!("gRPC server error: {}", e);
            }
        }
    }
    Ok(())
}
