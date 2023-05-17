use sqlx::Row;
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: Option<u32>, // Optional id field to be filled during database operations
    pub username: String,
    pub password: String, // Added field for the hashed password
}

