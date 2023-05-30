use serde::{Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub struct EmbeddingEntry {
    pub start_byte: i64,
    pub end_byte: i64,
    pub embedding: Vec<f64>,
}