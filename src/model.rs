#![allow(unused)]

use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::*;

// Data structures

#[derive(Debug, Clone, Serialize)]
pub struct Todo {
    id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    fetch: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct PostTodo {
    title: String,
    description: Option<String>,
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
            completed: false,
        }
    }
}

// Model controller

#[derive(Clone)]
pub struct ModelController {
    todos: Arc<Mutex<HashMap<Uuid, Todo>>>,
}

impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self { todos: Arc::default() })
    }

    pub async fn get_todos(&self, pagination: Pagination) -> Result<Vec<Todo>> {
        let todos = self.todos.lock()
            .map_err(|_| Error::DatabaseInternalFatal)?
            .iter()
            .skip(pagination.fetch.unwrap_or(0))
            .take(pagination.limit.unwrap_or(usize::MAX))
            .map(|todo| todo.1.clone())
            .collect();

        Ok(todos)
    }

    pub async fn post_todo(&self, todo: PostTodo) -> Result<Todo> {
        let mut todos = self.todos.lock()
            .map_err(|_| Error::DatabaseInternalFatal)?;

        let todo: Todo = todo.into();
        todos.insert(todo.id, todo.clone());

        Ok(todo.into())
    }

    pub async fn patch_todo(&self, id: Uuid, info: PatchTodo) -> Result<Todo> {
        let mut todos = self.todos.lock()
            .map_err(|_| Error::DatabaseInternalFatal)?;

        let todo = todos.get_mut(&id)
            .ok_or(Error::PatchTodoNotFound { id })?;
        todo.patch(info);

        Ok(todo.clone())
    }

    pub async fn delete_todo(&self, id: Uuid) -> Result<Todo> {
        let mut todos = self.todos.lock()
            .map_err(|_| Error::DatabaseInternalFatal)?;

        todos.remove(&id)
            .ok_or(Error::DeleteTodoNotFound { id })
    }
}

impl ModelController {

}
