use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Task failed: {0}")]
    TaskFailed(String),

    #[error("External tool error ({tool}): {msg}")]
    ExternalTool { tool: String, msg: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({
            "error": -1,
            "msg": self.to_string(),
            "data": null,
        });
        (StatusCode::OK, axum::Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
