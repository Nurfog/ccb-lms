use actix_web::{dev::Payload, FromRequest, HttpRequest};
use std::env;
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation};

pub mod middleware;
pub mod models;

pub use models::{Claims, User, UserRole};

/// Estructura que representa al usuario autenticado a través del token JWT.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: UserRole,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            std::future::ready(Ok(user.clone()))
        } else {
            std::future::ready(Err(actix_web::error::ErrorUnauthorized("Not authenticated or invalid token")))
        }
    }
}
