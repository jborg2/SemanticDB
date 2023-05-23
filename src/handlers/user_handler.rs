use actix_web::{web, HttpResponse, Responder};
use sqlx::{SqlitePool, Error};
use crate::models::user::User;
use crate::models::project::Project;
use crate::models::user_project::UserProject;
use crate::models::credentials::Credentials;
use crate::models::claims::Claims;
use crate::models::token_response::TokenResponse;


use sqlx::Acquire;
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use std::fs;

pub async fn add_user(db_pool: web::Data<SqlitePool>, new_user: web::Json<User>) -> impl Responder {
    let mut conn = db_pool.acquire().await.unwrap();
    let hashed_password = hash(&new_user.password, DEFAULT_COST).unwrap();

    let result = sqlx::query(
        r#"
        INSERT INTO users (username, hashed_password)
        VALUES (?, ?)
        "#,
    )
    .bind(&new_user.username)
    .bind(&hashed_password)
    .execute(&mut conn)
    .await;

    match result {
        Ok(_) => {
            HttpResponse::Ok().json(new_user.0)
        }, 
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

pub async fn login(
    db_pool: web::Data<SqlitePool>,
    credentials: web::Json<Credentials>,
) -> impl Responder {
    let mut conn = db_pool.acquire().await.unwrap();

    let row: Result<(i64,String,), sqlx::Error> = sqlx::query_as(
        r#"
        SELECT id, hashed_password 
        FROM users 
        WHERE username = ?
        "#
    )
    .bind(&credentials.username)
    .fetch_one(&mut conn)
    .await;
    
    match row {
        Ok((id, hashed_password,)) => {
            let is_password_correct = verify(&credentials.password, &hashed_password).unwrap();
    
            if is_password_correct {
                // Set up JWT claims
                let my_claims = Claims {
                    userID: id,
                    sub: credentials.username.clone(),
                    exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
                };

                let key = "temporary-test-key"; // TODO: load from environment variables or configuration file.

                let token = encode(
                    &Header::new(Algorithm::HS512),
                    &my_claims,
                    &EncodingKey::from_secret(key.as_ref()),
                )
                .unwrap();

                HttpResponse::Ok().json(TokenResponse { token })
            } else {
                HttpResponse::Unauthorized().body("Invalid username or password")
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Something went wrong"),
    }
}

pub async fn create_user(
    db_pool: web::Data<SqlitePool>,
    reset_password_credentials: web::Json<Credentials>
) -> impl Responder {
    
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::post().to(add_user))
    );

    cfg.service(
        web::resource("/login")
            .route(web::post().to(login))
    );
    
}