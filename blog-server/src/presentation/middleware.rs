use crate::infrastructure::jwt::JwtService;
use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req.app_data::<actix_web::web::Data<Arc<JwtService>>>() {
        Some(s) => s,
        None => {
            let err = ErrorUnauthorized("JWT service not configured");
            return Err((err, req));
        }
    };
    match jwt_service.verify_token(credentials.token()) {
        Ok(claims) => {
            let user = AuthenticatedUser {
                user_id: claims.sub,
            };
            req.extensions_mut().insert(user);
            Ok(req)
        }
        Err(_) => {
            let err = ErrorUnauthorized("Invalid token");
            Err((err, req))
        }
    }
}
