use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct UserProject {
    pub user_id: i64,
    pub project_id: i64,
    pub permission_type: String
}