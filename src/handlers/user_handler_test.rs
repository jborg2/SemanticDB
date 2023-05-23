#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{web, App, Responder, HttpResponse};
    use actix_web::http::StatusCode;
    use sqlx::{Sqlite, SqlitePool};
    use crate::handlers::user_handler::*;
    use crate::models::user::User;
    use crate::models::user_response::DatabaseUser;
    use crate::models::credentials::Credentials;
    use crate::models::reset_password_credentials::ResetPasswordCredentials;
    use tokio::fs::read_to_string; // Imported read_to_string

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
        // Read SQL commands from file
        let sql_commands = read_to_string("init.sql").await.expect("Could not read SQL file");
    
        // Execute SQL commands
        sqlx::query(&sql_commands).execute(&pool).await.expect("Could not execute SQL commands");
    
        pool
    }

    #[actix_rt::test]
    async fn test_add_user() {
        let pool = setup_db().await;

        let new_user = User {
            id: None,
            username: String::from("test_user"),
            password: String::from("password"),
        };

        let result = add_user(web::Data::new(pool.clone()), web::Json(new_user)).await;

        assert_eq!(result.status(), StatusCode::OK);
        
        // Query the database to check if the user was added
        let user = sqlx::query_as::<_, DatabaseUser>("SELECT * FROM users WHERE username = 'test_user'")
            .fetch_one(&pool)
            .await
            .expect("Failed to query database.");
        
        assert_eq!(user.username, "test_user");


        const verified = verify(String::from("password"), &user.hashed_password).unwrap();

        assert_eq!(verified, true);
    }

    /*
    #[actix_rt::test]
    async fn test_login() {
        let pool = setup_db().await;

        // Add a user before testing login
        add_test_user(web::Data::new(pool)).await;

        let credentials = Credentials {
            username: String::from("test_user"),
            password: String::from("password"),
        };

        let result = login(web::Data::new(pool), web::Json(credentials)).await;
        let response = result.downcast::<HttpResponse>().unwrap();
        
        assert_eq!(response.status(), 200);
        // Check the response body if necessary.
    }

    #[actix_rt::test]
    async fn test_reset_password() {
        let pool = setup_db().await;

        // Add a user before testing reset password
        add_test_user(web::Data::new(pool)).await;

        let reset_password_credentials = ResetPasswordCredentials {
            username: String::from("test_user"),
            password: String::from("password"),
            new_password: String::from("new_password"),
        };

        let result = reset_password(web::Data::new(pool), web::Json(reset_password_credentials)).await;
        let response = result.downcast::<HttpResponse>().unwrap();
        
        assert_eq!(response.status(), 200);
        // Check the response body if necessary.
    }
    */
}