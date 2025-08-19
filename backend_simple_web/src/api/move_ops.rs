// backend_simple_web/src/api/move_ops.rs
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use rocket::tokio::fs;
use std::path::{Path, PathBuf};

use crate::auth::Admin;
use super::{ROOT, clean};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FileMove {
    from: String,
    to: String,
}

/// For files/dirs that must already exist on disk.
async fn resolve_src(rel: &str) -> Result<PathBuf, Status> {
    let cleaned = clean(rel);
    if cleaned.is_empty() {
        return Err(Status::BadRequest);
    }

    let full = Path::new(ROOT).join(&cleaned);
    let canon = fs::canonicalize(&full)
        .await
        .map_err(|_| Status::BadRequest)?;
    let root_canon = fs::canonicalize(ROOT)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !canon.starts_with(&root_canon) {
        return Err(Status::BadRequest);
    }
    Ok(canon)
}

/// For a new or moved-to path: ensure its parent is valid, but allow the file itself to not exist.
async fn resolve_dst(rel: &str) -> Result<PathBuf, Status> {
    let cleaned = clean(rel);
    if cleaned.is_empty() {
        return Err(Status::BadRequest);
    }

    let full = Path::new(ROOT).join(&cleaned);
    let parent = full.parent().ok_or(Status::BadRequest)?;

    let parent_canon = fs::canonicalize(parent)
        .await
        .map_err(|_| Status::BadRequest)?;
    let root_canon = fs::canonicalize(ROOT)
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !parent_canon.starts_with(&root_canon) {
        return Err(Status::BadRequest);
    }

    Ok(full)
}

/// Move a file or directory
/// ### Arguments:
/// - `from` (required): relative path inside the public site
/// - `to` (required): relative path inside the public site
/// ### Examples:
/// - POST /api/move  JSON ```{"from":"old.html","to":"new.html"}```
#[post("/move", data = "<payload>")]
pub async fn move_entry(payload: Json<FileMove>, _admin: Admin) -> Result<Status, Status> {
    let src = resolve_src(&payload.from).await?;
    let dst = resolve_dst(&payload.to).await?;

    // Prevent moving a directory *inside itself* (would loop forever)
    if src.is_dir() && dst.starts_with(&src) {
        return Err(Status::BadRequest);
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).await.ok();
    }

    fs::rename(&src, &dst)
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Status::Ok)
}