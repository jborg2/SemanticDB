use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}
