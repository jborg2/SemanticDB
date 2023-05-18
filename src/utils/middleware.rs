use actix_service::{Service, Transform, forward_ready};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage, HttpResponse, error};
use futures::future::{ok, Either, ready, Future};
use futures::FutureExt; // Importing FutureExt trait for 'boxed_local' function
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::models::claims::Claims;
use chrono::prelude::*;

pub struct JwtMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddleware<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response  = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    forward_ready!(service);

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
                        self.service.call(req).boxed_local()
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
