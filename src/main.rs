#![allow(unused)]

mod model;
mod error;

use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, routing::{get, patch}, Json, Router};
use model::{ModelController, Pagination, PatchTodo, PostTodo};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::error::*;

#[tokio::main]
async fn main() -> Result<()> {
    let db = ModelController::new().await?;

    let app = Router::new()
    .route("/todos", get(handle_get).post(handle_post))
    .route("/todos/{id}", patch(handle_patch).delete(handle_delete))
    .with_state(db);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    log("Server start", "main()").await;
    axum::serve(listener, app)
    .await
    .unwrap();

    Ok(())
}

async fn log(what: &'static str, whence: &'static str) {
    println!("->> {:<16}\t{}", what, whence);
}

async fn handle_get(
    State(db): State<ModelController>, 
    Json(pagination): Json<Pagination>
) -> impl IntoResponse {
    let todos = db.get_todos(pagination).await;

    Json(todos)
}

async fn handle_post(
    State(db): State<ModelController>, 
    Json(todo): Json<PostTodo>
) -> impl IntoResponse {
    let todo = db.post_todo(todo).await;

    (StatusCode::CREATED, Json(todo))
}

async fn handle_patch(
    State(db): State<ModelController>, 
    Path(id): Path<Uuid>, 
    Json(info): Json<PatchTodo>
) -> impl IntoResponse {
    let todo = db.patch_todo(id, info).await;

    Json(todo)
}

async fn handle_delete(
    State(db): State<ModelController>, 
    Path(id): Path<Uuid>
) -> impl IntoResponse {
    let todo = db.delete_todo(id).await;
    
    Json(todo)
}

