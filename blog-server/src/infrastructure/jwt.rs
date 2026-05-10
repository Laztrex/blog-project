//! Модуль работы с JWT (JSON Web Tokens).
//! Содержит структуры Claims и JwtService для генерации и проверки токенов.

use crate::domain::error::DomainError;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: usize,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_hours: i64,
}

/// Создаёт новый сервис JWT.
/// Arguments:
/// `secret` – секретный ключ для подписи (не менее 32 символов).
/// `expiry_hours` – время жизни токена в часах.
impl JwtService {
    pub fn new(secret: &str, expiry_hours: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiry_hours,
        }
    }

    /// Генерирует новый JWT-токен для пользователя.
    /// Arguments:
    /// `user_id` – идентификатор пользователя.
    /// `username` – имя пользователя.
    ///
    /// Return:
    /// JWT-токен как строка.
    pub fn generate_token(&self, user_id: i64, username: &str) -> Result<String, DomainError> {
        let expiration = Utc::now() + Duration::hours(self.expiry_hours);
        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            exp: expiration.timestamp() as usize,
        };
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Проверяет JWT-токен и извлекает claims.
    /// Return:
    /// Claims при успехе, иначе ошибка DomainError::Jwt.
    pub fn verify_token(&self, token: &str) -> Result<Claims, DomainError> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}
