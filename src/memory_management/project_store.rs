extern crate blas;
use blas::{ddot, dnrm2};
use std::cmp::min;
use std::collections::HashMap;
use sqlx::{SqlitePool};
use sqlx::Acquire;
use serde::Deserialize;
use actix_web::{web, Error, HttpResponse};

#[derive(Clone)]
pub struct Embedding {
    pub embedding: Vec<f64>,
    pub start_byte: i64,
    pub end_byte: i64,
    pub file_id: i64,
}

pub struct ProjectStore {
    pub name: String,
    pub in_memory: bool,
    pub project_id: i64,
    pub file_ids: Vec<i64>,
    pub embeddings: Vec<Embedding>,
    pub vp_tree: Option<vpsearch::Tree<Embedding>>,
}

impl vpsearch::MetricSpace for Embedding {
    type UserData = ();
    type Distance = f64;
    fn distance(&self, other: &Self, _: &Self::UserData) -> f64 {
        let a = &self.embedding;
        let b = &other.embedding;
        let n = min(a.len(), b.len());
        let dot_product = unsafe { ddot(n as i32, &a, 1, &b, 1) };
        let a_magnitude = unsafe { dnrm2(n as i32, &a, 1) };
        let b_magnitude = unsafe { dnrm2(n as i32, &b, 1) };
        dot_product / (a_magnitude * b_magnitude)
    }
}

impl ProjectStore {
    pub fn new(name: String, project_id: i64, file_ids: Vec<i64>, in_memory: bool) -> ProjectStore {
        let mut embeddings = Vec::<Embedding>::new();
        let mut store = ProjectStore {
            name: name,
            project_id: project_id,
            file_ids: file_ids,
            in_memory: in_memory,
            embeddings: embeddings,
            vp_tree: None
        };

        store.vp_tree = Some(vpsearch::Tree::new(&store.embeddings));
        store
    }

    pub fn get_knn(&self, embedding: &Embedding, k: usize) -> Vec<vpsearch::Neighbor<Embedding>> {
        self.vp_tree.as_ref().unwrap().search(&embedding, k, &())
    }
}


