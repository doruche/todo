#![allow(unused)]

use serde::Serialize;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Serialize)]
pub enum Error {
    EnvVarConfigError,
    DatabaseInternalFatal,
    DatabaseConnectError,
    PatchTodoNotFound { id: Uuid },
    DeleteTodoNotFound { id: Uuid },
}