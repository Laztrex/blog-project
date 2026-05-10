use std::env;

/// Конфигурация приложения, загружаемая из переменных окружения.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_port: u16,
    pub grpc_port: u16,
    pub jwt_expiry_hours: i64,
    pub db_max_connections: u32,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            http_port: env::var("HTTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            grpc_port: env::var("GRPC_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50051),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(24),
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        }
    }
}
