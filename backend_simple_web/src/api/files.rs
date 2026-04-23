// backend_simple_web/src/api/files.rs
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::fs;
use std::path::{Path, PathBuf};

use prisma_auth::backend::AuthGuard as Admin;
use super::error::AppError;
use super::path::ValidatedPath;
use super::{ROOT, clean};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FileEntry {
    path: String,
    is_dir: bool,
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct FileBody {
    content: String,
}

// ------------- LIST FILES ---------------------------------------------------
/// List files in a directory
/// ### Arguments:
/// - `path` (optional): relative path inside the public site
/// ### Examples:
/// - GET /api/files               -> list ROOT
/// - GET /api/files?path=img/logo -> list ./img/logo
#[get("/files?<path>")]
pub async fn list_files(path: Option<String>, _admin: Admin) -> Json<Vec<FileEntry>> {
    let dir_path = match path.map(|p| clean(&p)) {
        Some(ref p) if !p.is_empty() => Path::new(ROOT).join(p),
        _ => PathBuf::from(ROOT),
    };

    let mut list = Vec::new();
    if let Ok(mut rd) = fs::read_dir(&dir_path).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            if let Ok(md) = entry.metadata().await {
                list.push(FileEntry {
                    path: entry
                        .path()
                        .strip_prefix(ROOT)
                        .unwrap()
                        .to_string_lossy()
                        .trim_start_matches('/')
                        .into(),
                    is_dir: md.is_dir(),
                });
            }
        }
    }
    Json(list)
}

// ------------- READ FILE ----------------------------------------------------
/// Read a file
/// ### Arguments:
/// - `path` (optional): relative path inside the public site
/// ### Examples:
/// - GET /api/file?path=index.html
#[get("/file?<path>")]
pub async fn get_file(path: Option<String>, _admin: Admin) -> Option<NamedFile> {
    let rel = path.map(|p| clean(&p)).filter(|p| !p.is_empty())?;
    let full = Path::new(ROOT).join(rel);
    NamedFile::open(full).await.ok()
}

// ------------- SAVE FILE ----------------------------------------------------
/// Save a file
/// ### Arguments:
/// - `path` (required): relative path inside the public site
/// - `content` (required): the file content
/// ### Examples:
/// - POST /api/file  JSON ```{"path":"css/app.css","content":"body{}"}```
#[post("/file?<path>", data = "<body>")]
pub async fn save_file(_admin: Admin, path: &str, body: Json<FileBody>) -> Result<Status, AppError> {
    let vp = ValidatedPath::new_destination(path)?;
    let full = vp.as_path();

    if fs::metadata(full)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        return Err(AppError::BadRequest("Target is a directory".into()));
    }

    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(full, &body.content).await?;
    Ok(Status::Ok)
}

// ------------- DELETE FILE / DIR -------------------------------------------
/// Delete a file or directory
/// ### Arguments:
/// - `path` (required): relative path inside the public site
/// ### Examples:
/// - DELETE /api/file?path=img/logo.png
#[delete("/file?<path>")]
pub async fn delete_file(path: Option<String>, _admin: Admin) -> Result<Status, AppError> {
    let rel = path.map(|p| clean(&p)).filter(|p| !p.is_empty())
        .ok_or_else(|| AppError::BadRequest("Path is required".into()))?;
    let vp = ValidatedPath::new(&rel)?;
    let full = vp.as_path();

    if fs::metadata(full)
        .await
        .ok()
        .map(|m| m.is_file())
        .unwrap_or(false)
    {
        fs::remove_file(full).await.map_err(|e| {
            AppError::Internal(format!("Failed to delete file {:?}: {}", full, e))
        })?;
    } else {
        fs::remove_dir_all(full).await.map_err(|e| {
            AppError::Internal(format!("Failed to delete directory {:?}: {}", full, e))
        })?;
    }
    Ok(Status::Ok)
}
