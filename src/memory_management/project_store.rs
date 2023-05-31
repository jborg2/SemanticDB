use std::collections::HashMap;
use sqlx::{SqlitePool};
use sqlx::Acquire;
use serde::Deserialize;
use actix_web::{web, Error, HttpResponse};

pub struct project_store {
    name: String,
    in_memory: bool,
    db_pool: web::Data<SqlitePool>,
    project_id: i64,
    file_ids: Vec<i64>,
    embeddings: HashMap<String, Vec<f64>>
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
struct file_id_wrapper {
    id: i64
}

impl project_store {
    fn new(db_pool: web::Data<SqlitePool>, name: String, project_id: i64, load_to_memory: bool) -> project_store {
        if load_to_memory {
            let mut store = project_store::new(db_pool.clone(), name, project_id, false);
            store.load_project_to_memory(db_pool);
            return store
        }

        project_store {
            name,
            in_memory: false,
            db_pool: db_pool,
            project_id: project_id,
            file_ids: Vec::new(),
            embeddings: HashMap::new()
        }
    }

    fn add_embedding(&mut self, file_id: i64, start_byte: i64, end_byte: i64, embedding: Vec<f64>) {
        let key = format!("{}:{}:{}", file_id, start_byte, end_byte);
        self.embeddings.insert(key, embedding);
    }

    async fn load_project_to_memory(&mut self, db_pool: web::Data<SqlitePool>) -> bool {
        let mut conn = db_pool.acquire().await.unwrap();
        let result: Result<Vec<file_id_wrapper>, sqlx::Error> = sqlx::query_as(
            r#"
            SELECT id
            FROM file_entry
            WHERE project_id = ?;
            "#,     
        )
        .bind(self.project_id)
        .fetch_all(&mut conn)
        .await;

        self.in_memory = true;
        true
    }

    fn remove_project_from_memory(&mut self) -> bool {
        self.in_memory = false;
        self.embeddings = HashMap::new();
        true
    }
}


