use actix_web::{web, HttpResponse, Responder};
use sqlx::{SqlitePool, Error};
use sqlx::Acquire;
use crate::models::project::Project;

pub async fn add_project(db_pool: web::Data<SqlitePool>, new_project: web::Json<Project>) -> impl Responder {
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

            let mut new_project_with_id = new_project.into_inner(); // Get the inner Project from Json<Project>
            new_project_with_id.id = Some(id); // Set the id to the newly inserted id

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
    project_id: web::Path<i32>,
) -> impl Responder {
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
        Err(e) => {
            eprintln!("Database error: {}", e); // Log the error
            HttpResponse::InternalServerError().body("Something went wrong")
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
}

