// backend_simple_web/src/api/error.rs
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ErrorBody {
    success: bool,
    message: String,
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let (status, msg) = match &self {
            AppError::BadRequest(m) => (Status::BadRequest, m.clone()),
            AppError::NotFound(m) => (Status::NotFound, m.clone()),
            AppError::Internal(m) => {
                error!("Internal error: {}", m);
                (Status::InternalServerError, m.clone())
            }
        };
        let body = Json(ErrorBody { success: false, message: msg });
        Response::build_from(body.respond_to(req)?)
            .status(status)
            .ok()
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound(e.to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::BadRequest(format!("Permission denied: {}", e)),
            _ => AppError::Internal(e.to_string()),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal(format!("JSON error: {}", e))
    }
}

impl From<git2::Error> for AppError {
    fn from(e: git2::Error) -> Self {
        AppError::Internal(format!("Git error: {}", e))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(m) => write!(f, "Bad request: {}", m),
            AppError::NotFound(m) => write!(f, "Not found: {}", m),
            AppError::Internal(m) => write!(f, "Internal error: {}", m),
        }
    }
}
