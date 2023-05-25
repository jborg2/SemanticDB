#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{web, App, http, test, dev::ServiceResponse};
    use actix_web::http::StatusCode;
    use sqlx::SqlitePool;
    use crate::handlers::project_handler::*;
    use crate::models::project::Project;
    use std::fs;
    use tokio::fs::read_to_string;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
        // Read SQL commands from file
        let sql_commands = read_to_string("init.sql").await.expect("Could not read SQL file");
    
        // Execute SQL commands
        sqlx::query(&sql_commands).execute(&pool).await.expect("Could not execute SQL commands");
    
        pool
    }

    #[actix_rt::test]
    async fn test_add_project() {
        let pool = setup_db().await;

        let new_project = Project {
            id: None,
            name: String::from("test_project"),
            description: String::from("test_description"),
        };

        let result = add_project(web::Data::new(pool.clone()), web::Json(new_project)).await;

        assert_eq!(result.status(), StatusCode::OK);

        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE name = 'test_project'")
            .fetch_one(&pool)
            .await
            .expect("Failed to query database.");
        
        assert_eq!(project.name, "test_project");
        assert_eq!(project.description, "test_description");
    }

    #[actix_rt::test]
    async fn test_get_project_by_id() {
        let pool = setup_db().await;
    
        let new_project = Project {
            id: None,
            name: String::from("test_project"),
            description: String::from("test_description"),
        };
    
        let result = add_project(web::Data::new(pool.clone()), web::Json(new_project)).await;
    
        assert_eq!(result.status(), StatusCode::OK);
    
        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE name = 'test_project'")
            .fetch_one(&pool)
            .await
            .expect("Failed to query database.");
        
        assert_eq!(project.name, "test_project");
        assert_eq!(project.description, "test_description");
    
        // Create a Path object from the project's ID
        let path = web::Path::from(project.id.unwrap());
    
        // Call the handler function with the database pool and Path object
        let result = get_project_by_id(web::Data::new(pool.clone()), path).await;
    
        assert_eq!(result.status(), StatusCode::OK);
    
        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE name = 'test_project'")
            .fetch_one(&pool)
            .await
            .expect("Failed to query database.");
        
        assert_eq!(project.name, "test_project");
        assert_eq!(project.description, "test_description");
    }
}

