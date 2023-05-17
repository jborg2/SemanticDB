use crate::models::project::Project
use serde::Serialize;

pub struct UserProjectDTO {
    pub project: ProjectDTO,
    pub permission: String
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct ProjectDTO {
    pub id: Option<i64>,
    pub name: String,
    pub description: String
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct UserDTO {
    pub id: Option<u32>, // Optional id field to be filled during database operations
    pub username: String,
    pub projects: Vec<UserProjectDTO>
}