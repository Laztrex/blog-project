use crate::data::user_repository::UserRepository;
use crate::domain::error::DomainError;
use crate::domain::user::{LoginUser, RegisterUser, User};
use crate::infrastructure::jwt::JwtService;

use std::sync::Arc;

use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(user_repo: Arc<dyn UserRepository>, jwt_service: Arc<JwtService>) -> Self {
        Self {
            user_repo,
            jwt_service,
        }
    }

    pub async fn register(&self, input: RegisterUser) -> Result<(String, User), DomainError> {
        if self
            .user_repo
            .find_by_username(&input.username)
            .await
            .is_ok()
        {
            return Err(DomainError::UserAlreadyExists);
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)
            .map_err(|e| DomainError::Argon2(e.to_string()))?
            .to_string();

        let user = self
            .user_repo
            .create(&input.username, &input.email, &password_hash)
            .await?;
        let token = self.jwt_service.generate_token(user.id, &user.username)?;
        Ok((token, user))
    }

    pub async fn login(&self, input: LoginUser) -> Result<(String, User), DomainError> {
        let user = self.user_repo.find_by_username(&input.username).await?;
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| DomainError::Argon2(e.to_string()))?;
        let argon2 = Argon2::default();
        argon2
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .map_err(|_| DomainError::InvalidCredentials)?;
        let token = self.jwt_service.generate_token(user.id, &user.username)?;
        Ok((token, user))
    }
}
