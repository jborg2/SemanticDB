use std::collections::HashMap;
use crate::memory_management::project_store::project_store;
use sqlx::{SqlitePool};
use sqlx::Acquire;
use serde::Deserialize;
use actix_web::{web, Error, HttpResponse};

pub struct ProjectManager {
    projects: HashMap<String, project_store>,
    dbPool: SqlitePool
}

impl ProjectManager {
    pub fn new(dbPool: SqlitePool) -> ProjectManager {
        ProjectManager {
            projects: HashMap::new()
            dbPool: dbPool
        }


    }

    fn add_project(&mut self, name: String, project_store: project_store) {
        self.projects.insert(name, project_store);
    }

    fn get_project(&mut self, name: String) -> Option<&mut project_store> {
        self.projects.get_mut(&name)
    }
}