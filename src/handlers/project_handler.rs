use actix_web::{web, Error, HttpResponse};
use sqlx::{SqlitePool};
use sqlx::Acquire;
use crate::models::project::Project;
use futures::io::AsyncWriteExt;
use std::fs;
use actix_multipart::{Field, Multipart};
use actix_web::http::header::ContentDisposition;
use actix_web::Responder;
use async_std::fs::File;
use async_std::prelude::*;  // Import prelude for write_all
use futures::TryStreamExt;
use serde::Deserialize;
use crate::memory_management::project_manager::ProjectManager;
use std::sync::{Arc, Mutex};

pub async fn add_project(project_manager: web::Data<Arc<Mutex<ProjectManager>>>, db_pool: web::Data<SqlitePool>, new_project: web::Json<Project>
) -> HttpResponse {
    let mut project_manager = project_manager.lock().unwrap();
    let mut conn = db_pool.acquire().await.unwrap();

    let mut transaction = conn.begin().await.unwrap(); // Start a new transaction

    let result = sqlx::query(
        r#"
        INSERT INTO projects (name, description)
        VALUES (?, ?);
        "#,
    )
    .bind(&new_project.name)
    .bind(&new_project.description)
    .execute(&mut transaction)
    .await;

    match result {
        Ok(_) => {
            let id: i64 = sqlx::query_scalar("SELECT LAST_INSERT_ROWID()").fetch_one(&mut transaction).await.unwrap();
            transaction.commit().await.unwrap(); // Commit the transaction
            project_manager.add_blank_project(id, new_project.name.clone());
            let mut new_project_with_id = new_project.into_inner(); // Get the inner Project from Json<Project>
            new_project_with_id.id = Some(id); // Set the id to the newly inserted id
            fs::create_dir(format!("./project_data/{}", id));
            HttpResponse::Ok().json(new_project_with_id) // Respond with the new project with id
        },
        Err(e) => {
            eprintln!("Database error: {}", e); // Log the error
            HttpResponse::InternalServerError().body("Something went wrong")
        },
    }
}

pub async fn get_project_by_id(
    db_pool: web::Data<SqlitePool>,
    project_id: web::Path<i64>,
) -> HttpResponse {
    let mut conn = db_pool.acquire().await.unwrap();
    let result: Result<Project, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT id, name, description FROM projects WHERE id = ?
        "#,
    )
    .bind(project_id.into_inner())
    .fetch_one(&mut conn)
    .await;

    match result {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                HttpResponse::NotFound().body("Project not found")
            }
            _ => {
                eprintln!("Database error: {}", e); // Log the error
                HttpResponse::InternalServerError().body("Something went wrong")
            }
        },
    }
}

async fn read_string(field: &mut Field) -> Option<String> {
    let bytes = field.try_next().await;

    if let Ok(Some(bytes)) = bytes {
        String::from_utf8(bytes.to_vec()).ok()
    } else {
        None
    }
}

pub async fn upload(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    mut payload: Multipart,
) -> Result<HttpResponse, actix_web::Error> {
    let mut upload_name = String::new();
    let mut file_path = String::from("");
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition(); //.unwrap();
        let field_name = content_disposition.get_name().unwrap();

        match field_name {
            "upload_name" => {
                if let Some(name) = read_string(&mut field).await { // pass `&mut field` instead of `&mut item`
                    upload_name = name;
                    println!("{}", upload_name)
                } else {
                    // Handle the case when read_string() returns None.
                    // You could return an error response, or you can just continue.
                }
            },
            "payload" => {
                file_path = format!("./project_data/{}/{}", id, sanitize_filename::sanitize(&upload_name));
                
                let mut f = async_std::fs::File::create(&file_path).await.unwrap();
        
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    async_std::io::WriteExt::write_all(&mut f, &data).await.unwrap();
                }
            },
            _ => {

            }
        } 
    }

    let mut conn = db_pool.acquire().await.unwrap();
    let result = sqlx::query(
        r#"
        INSERT INTO file_entry (name, path, project_id)
        VALUES (?, ?, ?);
        "#,     
    )
    .bind(upload_name)
    .bind(file_path)
    .bind(*id)
    .execute(&mut conn)
    .await;
    Ok(HttpResponse::Ok().into())
}

pub async fn get_files_by_project_id(
    db_pool: web::Data<SqlitePool>,
    project_id: web::Path<i64>,
) -> HttpResponse {
    let mut conn = db_pool.acquire().await.unwrap();
    let result: Result<Vec<crate::models::file::File>, sqlx::Error> = sqlx::query_as(
        r#"
        SELECT id, name, path, project_id FROM file_entry WHERE project_id = ?
        "#,
    )
    .bind(project_id.into_inner())
    .fetch_all(&mut conn)
    .await;

    match result {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                HttpResponse::NotFound().body("Project not found")
            }
            _ => {
                eprintln!("Database error: {}", e); // Log the error
                HttpResponse::InternalServerError().body("Something went wrong")
            }
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/projects")
            .route(web::post().to(add_project))
    );

    cfg.service(
        web::resource("/projects/{id}")
            .route(web::get().to(get_project_by_id))
    );

    cfg.service(
        web::resource("/projects/{id}/file")
            .route(web::put().to(upload))
    );

    cfg.service(
        web::resource("/projects/{id}/files")
            .route(web::get().to(get_files_by_project_id))
    );
}
