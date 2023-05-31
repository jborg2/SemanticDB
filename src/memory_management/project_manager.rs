use std::collections::HashMap;
use crate::memory_management::project_store::ProjectStore;
use sqlx::{SqlitePool};
use sqlx::Acquire;
use serde::Deserialize;
use actix_web::{web, Error, HttpResponse};

pub struct ProjectManager {
    projects: HashMap<String, ProjectStore>,
    dbPool: SqlitePool
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
struct ProjectQueryResult {
    id: i64,
    auto_load: bool,
    name: String,
    file_ids: String
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
struct EmbeddingResultQuery {
    file_id: i64,
    start_byte: i64,
    end_byte: i64,
    embedding: Vec<u8>
}

impl ProjectManager {
    pub fn new(dbPool: SqlitePool) -> ProjectManager {
        ProjectManager {
            projects: HashMap::new(),
            dbPool: dbPool
        }


    }

    pub async fn init_projects(&mut self) {
        let mut conn = self.dbPool.acquire().await.unwrap();
        let result: Result<Vec<ProjectQueryResult>, sqlx::Error> = sqlx::query_as(
            r#"
            SELECT projects.id, projects.auto_load, projects.name, GROUP_CONCAT(file_entry.id) as file_ids
            FROM projects
            LEFT JOIN file_entry ON projects.id = file_entry.project_id
            GROUP BY projects.id;            
            "#,     
        )
        .fetch_all(&mut conn)
        .await;
        
        for project in result.unwrap() {
            println!("Loading Project id to memory: {}", project.name);
            let mut conn = self.dbPool.acquire().await.unwrap();
            let mut project_hashmap = HashMap::new();

            let result: Result<Vec<EmbeddingResultQuery>, sqlx::Error> = sqlx::query_as(
                r#"
                SELECT 
                    projects.id, 
                    projects.name, 
                    file_entry.id as file_id,
                    file_embedding.start_byte,
                    file_embedding.end_byte,
                    file_embedding.embedding
                FROM projects
                JOIN file_entry ON projects.id = file_entry.project_id
                JOIN file_embedding ON file_entry.id = file_embedding.file_id
                WHERE projects.id = ?
                "#,     
            )
            .bind(project.id)
            .fetch_all(&mut conn)
            .await;            
            let file_ids: Vec<i64> = project.file_ids.split(",").map(|x| x.parse::<i64>().unwrap()).collect();
            for embedding in result.unwrap() {
                let embedding_str = String::from_utf8(embedding.embedding).unwrap();
                let data: Vec<f64> = serde_json::from_str(&embedding_str).unwrap();
                let key = format!("{}-{}-{}", embedding.file_id, embedding.start_byte, embedding.end_byte);
                project_hashmap.insert(key, data);
            }

            /*
            let project_store = ProjectStore {
                project_id: project.id,
                name: project.name.clone(),
                in_memory: true,
                file_ids: file_ids,
                embeddings: project_hashmap
            };

            self.add_project(project.name, project_store);
            */
        }

    }

    fn add_project(&mut self, name: String, project_store: ProjectStore) {
        self.projects.insert(name, project_store);
    }

    fn get_project(&mut self, name: String) -> Option<&mut ProjectStore> {
        self.projects.get_mut(&name)
    }
}