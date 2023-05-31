use std::collections::HashMap;
use sqlx::{SqlitePool};
use sqlx::Acquire;
use serde::Deserialize;
use actix_web::{web, Error, HttpResponse};

pub struct project_store {
    name: String,
    in_memory: bool,
    project_id: i64,
    file_ids: Vec<i64>,
    embeddings: HashMap<String, Vec<f64>>
}


