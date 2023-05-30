use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct File {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub project_id: i64
}