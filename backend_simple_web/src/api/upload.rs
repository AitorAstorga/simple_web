// backend_simple_web/src/api/upload.rs
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::tokio::fs;
use std::path::{Component, Path, PathBuf};

use crate::auth::Admin;
use super::{ROOT, clean};

#[derive(FromForm)]
pub struct Upload<'r> {
    #[field(name = "files")]
    files: Vec<TempFile<'r>>,
    #[field(name = "base_path")]
    base_path: Option<String>,
}

/// Sanitize and normalize a path
/// ### Arguments:
/// - `path` (required): the path to sanitize
/// ### Returns:
/// - `Result<String, Status>`: the sanitized path or an error
fn sanitize_path(path: &str) -> Result<String, Status> {
    // Remove any null bytes
    if path.contains('\0') {
        error!("❌ path contains null bytes");
        return Err(Status::BadRequest);
    }
    
    // Check for invalid characters that could be dangerous
    let dangerous_chars = ['<', '>', ':', '"', '|', '?', '*'];
    if path.chars().any(|c| dangerous_chars.contains(&c)) {
        error!("❌ path contains dangerous characters");
        return Err(Status::BadRequest);
    }
    
    // Normalize path separators (convert backslashes to forward slashes)
    let normalized = path.replace('\\', "/");
    
    Ok(normalized)
}

/// Upload multiple files or folders at once
/// ### Arguments:
/// - `files` (required): the files to upload
/// - `base_path` (optional): relative path inside the public site
/// ### Examples:
/// - POST /api/upload  JSON ```{"files":[],"base_path":"img"}```
/// - POST /api/upload  JSON ```{"files":[],"base_path":"img/logo.png"}```
#[post("/upload", data = "<payload>")]
pub async fn upload(mut payload: Form<Upload<'_>>, _admin: Admin) -> Result<Status, Status> {
    // The directory in which the user wants to drop everything
    let raw_base = payload.base_path.take().unwrap_or_default();
    let base = clean(&raw_base);
    if base.contains("..") || base.starts_with('/') {
        error!("❌ invalid base path: {}", base);
        return Err(Status::BadRequest);
    }
    for file in payload.files.iter_mut() {
        let file_name = match file.raw_name() {
            Some(fname) => fname,
            None => {
                info!("⚠️ skipping unnamed part");
                continue;
            }
        };
        
        // Get the full path including directories
        let raw_name = file_name.dangerous_unsafe_unsanitized_raw().as_str();
        debug!("📤 full path: {}", raw_name);
        debug!("📤 base path: {}", base);
        
        let sanitized_name = sanitize_path(raw_name)?;

        // Validate: no absolute, no ".."
        let rel_path = Path::new(&sanitized_name);
        if rel_path.is_absolute() || rel_path.components().any(|c| c == Component::ParentDir) {
            error!("❌ invalid upload path: {}", raw_name);
            return Err(Status::BadRequest);
        }

        // Compose: ROOT / base_path / raw_name
        let mut full = PathBuf::from(ROOT);
        if !base.is_empty() {
            full.push(&base);
        }
        full.push(rel_path);

        info!("✅ persisting upload to {:?}", full);

        // Ensure the directory tree exists
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                error!("❌ failed to create {:?}: {:?}", parent, e);
                Status::InternalServerError
            })?;
        }

        // Write the file out
        file.persist_to(&full).await.map_err(|e| {
            error!("❌ persist_to {:?} failed: {:?}", full, e);
            Status::InternalServerError
        })?;
    }

    info!("✅ all uploads processed successfully");
    Ok(Status::Ok)
}