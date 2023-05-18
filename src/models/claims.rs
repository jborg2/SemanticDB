use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub userID: i64,
    pub sub: String,
    pub exp: usize,
}