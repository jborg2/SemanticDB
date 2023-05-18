use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage, error};
use futures::future::{ok, Either, ready, Ready, Future};
use futures::FutureExt;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::models::claims::Claims;
use chrono::prelude::*;

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>> + 'static>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareService { service }).boxed_local()
    }
}

pub struct JwtMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("API call middleware invoked");
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_token) = auth_header.to_str() {
                let token = auth_token.trim_start_matches("Bearer ").to_string();
            
                // Load the secret key from environment variables or configuration file
                let key = "temporary-test-key"; // TODO: Replace with the actual secret key
                
                // Decode and verify the token
                match decode::<Claims>(
                    &token,
                    &DecodingKey::from_secret(key.as_ref()),
                    &Validation::new(Algorithm::HS512),
                ) {
                    Ok(token_data) => {
                        // Check if the token is expired
                        if token_data.claims.exp < Utc::now().timestamp() as usize {
                            return ready(Err(error::ErrorUnauthorized("Token expired"))).boxed_local();
                        }
                        
                        // Add the authenticated user ID to the request extensions
                        req.extensions_mut().insert(token_data.claims.userID); 

                        // Continue processing the request with the user ID
                        Box::pin(self.service.call(req))
                    }
                    Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))).boxed_local(),
                }
            } else {
                ready(Err(actix_web::error::ErrorUnauthorized("Invalid Authorization header format"))).boxed_local()
            }
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("Missing Authorization header"))).boxed_local()
        }
    }
}
