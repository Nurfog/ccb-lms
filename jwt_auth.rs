use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage,
};
use futures::future::{ok, Ready};
use std::future::{ready, Future};
use std::pin::Pin;
use crate::models::{Claims, User, UserRole};

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Student,
    Instructor,
    Admin,
}

pub struct JwtMiddleware {
    required_roles: Vec<Role>,
}

impl JwtMiddleware {
    pub fn new(required_roles: Vec<Role>) -> Self {
        Self { required_roles }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareService {
            service,
            required_roles: self.required_roles.clone(),
        })
    }
}

pub struct JwtMiddlewareService<S> {
    service: S,
    required_roles: Vec<Role>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Aquí iría la lógica de validación del token que estaba en `AuthenticatedUser::from_request`
        // Por simplicidad, la omitimos por ahora para centrarnos en la estructura.
        // En un paso futuro, moveremos la lógica aquí.
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}