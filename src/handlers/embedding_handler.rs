use actix_web::{web, Error, HttpResponse};
use sqlx::{SqlitePool};
use sqlx::Acquire;
use reqwest::{self, header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION}};
use serde::{Deserialize, Serialize};
use crate::models::file::File;
use serde_json::json;
use tokio;
use std::fs;
use std::io::SeekFrom;
use std::io::prelude::*;
use futures::future::join_all;
use crate::models::embedding_entry::EmbeddingEntry;
use crate::memory_management::project_manager::ProjectManager;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub struct Embedding {
    embedding: Vec<f64>,
    index: i32,
    object: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    prompt_tokens: i32,
    total_tokens: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    data: Vec<Embedding>,
    model: String,
    object: String,
    usage: Usage,
    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    input: String,
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct similiar_text_request {
    text: String,
    project_id: i64
}

pub async fn get_embedding(input_string: String) -> Result<Response, reqwest::Error> {
    let api_key = std::env::var("OPENAI_API_TOKEN").expect("OPENAI_API_TOKEN must be set.");
    let data = Request {
        input: input_string,
        model: String::from("text-embedding-ada-002"),
    };

    let client = reqwest::Client::new();
    let mut res = client.post("https://api.openai.com/v1/embeddings")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .json(&data)
        .send()
        .await?;
        
    if !res.status().is_success() {
        let server_error: Result<Response, _> = res.json().await;
        return server_error;
    }
    
    let response_body: Result<Response, _> = res.json().await;
    response_body
}

//pub async fn run_embeddings_and_store(db_pool: web::Data<SqlitePool>, input_string: String, )

fn read_bytes_range(mut file: &std::fs::File, start: u64, end: u64) -> Vec<u8> {
    let mut buffer = vec![0; (end - start) as usize]; // Create a buffer to hold the bytes

    file.seek(std::io::SeekFrom::Start(start));
    file.read_exact(&mut buffer); // Read the specified range of bytes into the buffer

    buffer
}

async fn embed_and_store(db_pool: web::Data<SqlitePool>, file: &std::fs::File, start: u64, end: u64, file_id: i64) -> Result<(), sqlx::Error> {
    let bytes = read_bytes_range(&file, start, end);
    let input_string = String::from_utf8_lossy(&bytes).to_string();
    match get_embedding(input_string).await {
        Ok(embedding) => {
            let embedding_data = embedding.data[0].embedding.clone();
            let data = match serde_json::to_vec(&embedding_data) {
                Ok(data) => data,
                Err(_) => return Err(sqlx::Error::Protocol("Failed to serialize embedding data".into())),
            };

            let mut conn = db_pool.acquire().await.unwrap();

            let mut transaction = conn.begin().await.unwrap(); // Start a new transaction
        
            let result = sqlx::query(
                r#"
                    INSERT INTO file_embedding (file_id, start_byte, end_byte, embedding) VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(file_id)
            .bind(start as i64)
            .bind(end as i64)
            .bind(&data)
            .execute(&mut transaction);
            
            match result.await {
                Ok(_) => {
                    transaction.commit().await?;
                    Ok(())
                },
                Err(e) => Err(e),
            }
            

        },
        Err(e) => {
            eprintln!("OpenAI error: {}", e); 
            Err(sqlx::Error::Protocol("Error while getting embedding".into()))
        }
    }
}

pub async fn embed_file(db_pool: web::Data<SqlitePool>, file_id: web::Path<i64>) -> HttpResponse {
    let mut conn = db_pool.acquire().await.unwrap();
    let api_key = std::env::var("OPENAI_API_TOKEN").expect("OPENAI_API_TOKEN must be set.");
    let result: Result<File, sqlx::Error> = sqlx::query_as(
        r#"
            SELECT id, name, path, project_id FROM file_entry WHERE id = ?
        "#,
    )
    .bind(file_id.into_inner())
    .fetch_one(&mut conn)
    .await;
    match result {
        Ok(file) => {
            let file_path = file.path;
            let path = std::path::Path::new(&file_path);
            let display = path.display();
            match std::fs::File::open(&file_path) {
                Err(why) => panic!("couldn't open {}: {}", display, why),
                Ok(mut file_data) => {
                    let chunk_size: u64 = 1024;
                    let metadata = match file_data.metadata() {
                        Err(why) => panic!("couldn't get metadata for {}: {}", display, why),
                        Ok(metadata) => metadata,
                    };
                    let file_size = metadata.len();
                    let mut start: u64 = 0;
                    let mut futures = Vec::new();
                    while start < file_size {
                        let end = std::cmp::min(start + chunk_size, file_size);
                        futures.push(embed_and_store(db_pool.clone(), &file_data, start, end, file.id));
                        start += chunk_size;
                    }
                    let results = join_all(futures).await;
                    for result in results {
                        match result {
                            Ok(_) => {
                                eprintln!("Successfully embedded chunk");
                            },
                            Err(e) => {
                                eprintln!("Database error: {}", e); 
                                return HttpResponse::InternalServerError().body("Something went wrong")
                            }
                        }
                    }
                    return HttpResponse::Ok().body("Successfully embedded file")
                },
            }
        },
        Err(e) => {
            eprintln!("Database error: {}", e); 
            HttpResponse::InternalServerError().body("Something went wrong")
        }
    }
}

pub async fn get_embeddings(db_pool: web::Data<SqlitePool>, file_id: web::Path<i64>) -> HttpResponse {
    let mut conn = db_pool.acquire().await.unwrap();
    let result: Result<Vec<(i64, i64, Vec<u8>)>, sqlx::Error> = sqlx::query_as(
        r#"
            SELECT start_byte, end_byte, embedding
            FROM file_embedding
            WHERE file_id = ?
        "#,
    )
    .bind(file_id.into_inner())
    .fetch_all(&mut conn)
    .await;

    match result {
        Ok(rows) => {
            let mut embeddings = Vec::new();
            for (start_byte, end_byte, blob) in rows {
                let embedding: Vec<f64> = match serde_json::from_slice(&blob) {
                    Ok(embedding) => embedding,
                    Err(e) => {
                        eprintln!("Failed to deserialize embedding: {}", e);
                        return HttpResponse::InternalServerError().body("Something went wrong");
                    },
                };
                embeddings.push(EmbeddingEntry { start_byte, end_byte, embedding });
            }
            HttpResponse::Ok().json(embeddings)
        },
        Err(e) => {
            eprintln!("Database error: {}", e); 
            HttpResponse::InternalServerError().body("Something went wrong")
        }
    }
}

pub async fn get_similiar_text(project_manager: web::Data<Arc<Mutex<ProjectManager>>>, similiar_text_request: web::Json<similiar_text_request>) -> HttpResponse  {
    let mut project_manager = project_manager.lock().unwrap();
    match get_embedding(similiar_text_request.text.clone()).await {
        Ok(embedding) => {
            let embedding = embedding.data[0].embedding.clone();
            let input_embedding = crate::memory_management::project_store::Embedding {
                embedding: embedding.clone(),
                start_byte: -1,
                end_byte: -1,
                file_id: -1
            };
    
            let in_str = String::from("test project 2");
    
            
            let most_similiar_index = project_manager.get_most_similiar_embedding(in_str, input_embedding);
    
            HttpResponse::Ok().json(most_similiar_index)
        }
        Err(e) => {
            eprintln!("OpenAI error: {}", e); 
            HttpResponse::InternalServerError().body("Something went wrong")
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/file/{id}/embed")
            .route(web::get().to(embed_file))
    );

    cfg.service(
        web::resource("/file/{id}/embeddings")
            .route(web::get().to(get_embeddings))
    );

    cfg.service(
        web::resource("/embeddings/similiar")
            .route(web::post().to(get_similiar_text))
    );
}
