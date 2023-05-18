mod utils;
mod models;
mod handlers;


use actix_web::{App, HttpServer, web};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, ConnectOptions};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use actix_service::Service;
use crate::utils::middleware::JwtMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // This will create a new SQLite database in the same directory as your Cargo.toml file.
    let database_url = "sqlite://db/embeddings_database.db";
    let pool: SqlitePool;

    // Check if the database file exists
    if !Path::new(&database_url[9..]).exists() {
        // Create a new SQLite database
        let options = SqliteConnectOptions::new()
            .filename(&database_url[9..])
            .create_if_missing(true);

        pool = SqlitePool::connect_with(options)
            .await
            .expect("Failed to create pool.");

        // Read init.sql file
        let mut file = File::open("init.sql").await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        // Execute init.sql
        sqlx::query(&contents)
            .execute(&pool)
            .await
            .expect("Failed to initialize database.");
    } else {
        pool = SqlitePool::connect(&database_url)
            .await
            .expect("Failed to create pool.");
    }


    HttpServer::new(move || {

        App::new()
            .data(web::JsonConfig::default().limit(10 * 1024 * 1024)) 
            .app_data(web::Data::new(pool.clone()))
            /*.wrap(JwtMiddleware)*/
            .configure(handlers::user_handler::init_routes)
            .configure(handlers::project_handler::init_routes)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
