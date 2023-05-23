use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordCredentials {
    pub username: String,
    pub password: String,
    pub new_password: String,
}