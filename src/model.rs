#![allow(unused)]

use std::{collections::HashMap, env, sync::{Arc, Mutex}, thread::panicking};

use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::{PgConnectOptions, PgPoolOptions}, prelude::FromRow, query, query_as, PgPool, Row};
use uuid::Uuid;

use crate::error::*;

// Data structures

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Todo {
    id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    offset: Option<usize>,
    fetch: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct PostTodo {
    title: String,
    description: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PatchTodo {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

impl Todo {
    fn patch(&mut self, info: PatchTodo) {
        info.title.map(|t| self.title = t);
        self.description = info.description;
        info.completed.map(|c| self.completed = c);
    }
}

impl From<PostTodo> for Todo {
    fn from(value: PostTodo) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: value.title,
            description: value.description,
            completed: value.completed.unwrap_or(false),
        }
    }
}

// Model controller

#[derive(Clone)]
pub struct ModelController {
    pool: PgPool,
}

impl ModelController {
    pub async fn new() -> Result<Self> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| Error::EnvVarConfigError)?;


        let pool = PgPool::connect_lazy(&url)
            .map_err(|_| Error::DatabaseConnectError)?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|_| Error::DatabaseConnectError)?;

        Ok(Self { pool })
    }

    pub async fn get_todos(&self, pagination: Pagination) -> Result<Vec<Todo>> {
        let get = "SELECT * FROM todos
        OFFSET $1 ROWS
        FETCH NEXT $2 ROWS ONLY";

        let todos = query_as::<_, Todo>(get)
            .bind(pagination.offset.unwrap_or(0) as i64)
            .bind(pagination.fetch.unwrap_or(i32::MAX as usize) as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| Error::DatabaseInternalFatal)?;

        Ok(todos)
    }

    pub async fn post_todo(&self, todo: PostTodo) -> Result<Todo> { // completed
        let todo: Todo = todo.into();

        let post = "INSERT INTO todos (id, title, description, completed)
        VALUES ($1, $2, $3, $4)";

        query(post)
            .bind(&todo.id)
            .bind(&todo.title)
            .bind(&todo.description)
            .bind(&todo.completed)
            .execute(&self.pool)
            .await
            .map_err(|_| Error::DatabaseInternalFatal)?;

        Ok(todo)
    }

    pub async fn patch_todo(&self, id: Uuid, info: PatchTodo) -> Result<Todo> {
        let patch = "UPDATE todos SET
        title = CASE WHEN $1 = true THEN $2 ELSE title END,
        description = CASE WHEN $3 = true THEN $4 ELSE description END,
        completed = CASE WHEN $5 = true THEN $6 ELSE completed END
        WHERE id = $7
        RETURNING *";

        let todo = query_as::<_, Todo>(patch)
            .bind(info.title.is_some())
            .bind(info.title)
            .bind(info.description.is_some())
            .bind(info.description)
            .bind(info.completed.is_some())
            .bind(info.completed.unwrap_or(false))
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::PatchTodoNotFound { id })?;

        Ok(todo)
    }

    pub async fn delete_todo(&self, id: Uuid) -> Result<Todo> {
        let delete = "DELETE FROM todos
        WHERE id = $1
        RETURNING *";

        let todo = query_as::<_, Todo>(delete)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| Error::DeleteTodoNotFound { id })?;

        Ok(todo)
    }
}

impl ModelController {
    fn init(&mut self) {

    }
}
