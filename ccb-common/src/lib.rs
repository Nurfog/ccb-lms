use actix_web::{dev::Payload, FromRequest, HttpRequest};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::Type;

/// Enum para los roles de usuario.
/// Se deriva de `sqlx::Type` para que sqlx pueda mapear el ENUM de PostgreSQL a este tipo de Rust.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Student,
    Instructor,
    Admin,
}

/// Estructura para las 'claims' (afirmaciones) del JWT.
/// La movemos aquí para que todos los servicios la entiendan.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (el ID del usuario)
    pub role: UserRole, // El rol del usuario
    pub exp: usize,  // Expiration time (timestamp)
}

/// Estructura que representa al usuario autenticado a través del token JWT.
/// Ahora vive en el crate común y puede ser usada por cualquier servicio.
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: UserRole,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str["Bearer ".len()..];
                    let jwt_secret =
                        env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());

                    if let Ok(token_data) = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(jwt_secret.as_ref()),
                        &Validation::default(),
                    ) {
                        if let Ok(user_id) = Uuid::parse_str(&token_data.claims.sub) {
                            return std::future::ready(Ok(AuthenticatedUser { id: user_id, role: token_data.claims.role }));
                        }
                    }
                }
            }
        }

        // Si algo falla, denegamos el acceso.
        std::future::ready(Err(actix_web::error::ErrorUnauthorized("Invalid or missing token")))
    }
}
