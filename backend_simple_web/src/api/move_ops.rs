// backend_simple_web/src/api/move_ops.rs
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use rocket::tokio::fs;

use prisma_auth::backend::AuthGuard as Admin;
use super::error::AppError;
use super::path::ValidatedPath;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FileMove {
    from: String,
    to: String,
}

/// Move a file or directory
/// ### Arguments:
/// - `from` (required): relative path inside the public site
/// - `to` (required): relative path inside the public site
/// ### Examples:
/// - POST /api/move  JSON ```{"from":"old.html","to":"new.html"}```
#[post("/move", data = "<payload>")]
pub async fn move_entry(payload: Json<FileMove>, _admin: Admin) -> Result<Status, AppError> {
    let src = ValidatedPath::existing(&payload.from).await?;
    let dst = ValidatedPath::new_destination(&payload.to)?;

    // Prevent moving a directory inside itself
    if src.as_path().is_dir() && dst.as_path().starts_with(src.as_path()) {
        return Err(AppError::BadRequest("Cannot move a directory inside itself".into()));
    }

    if let Some(parent) = dst.as_path().parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            AppError::Internal(format!("Failed to create directory {:?}: {}", parent, e))
        })?;
    }

    fs::rename(src.as_path(), dst.as_path())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to move: {}", e)))?;

    Ok(Status::Ok)
}
