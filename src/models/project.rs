use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub description: String
}