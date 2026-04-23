// backend_simple_web/src/api/upload.rs
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::tokio::fs;

use prisma_auth::backend::AuthGuard as Admin;
use super::error::AppError;
use super::path::ValidatedPath;
use super::clean;

#[derive(FromForm)]
pub struct Upload<'r> {
    #[field(name = "files")]
    files: Vec<TempFile<'r>>,
    #[field(name = "base_path")]
    base_path: Option<String>,
}

/// Upload multiple files or folders at once
/// ### Arguments:
/// - `files` (required): the files to upload
/// - `base_path` (optional): relative path inside the public site
/// ### Examples:
/// - POST /api/upload  JSON ```{"files":[],"base_path":"img"}```
/// - POST /api/upload  JSON ```{"files":[],"base_path":"img/logo.png"}```
#[post("/upload", data = "<payload>")]
pub async fn upload(mut payload: Form<Upload<'_>>, _admin: Admin) -> Result<Status, AppError> {
    // Validate base path if provided
    let raw_base = payload.base_path.take().unwrap_or_default();
    let base = clean(&raw_base);
    if !base.is_empty() {
        // Validate base path with ValidatedPath
        ValidatedPath::new(&base)?;
    }

    for file in payload.files.iter_mut() {
        let file_name = match file.raw_name() {
            Some(fname) => fname,
            None => {
                info!("skipping unnamed part");
                continue;
            }
        };

        // Get the full path including directories
        let raw_name = file_name.dangerous_unsafe_unsanitized_raw().as_str();
        debug!("full path: {}", raw_name);
        debug!("base path: {}", base);

        // Normalize backslashes and validate the file name
        let sanitized_name = raw_name.replace('\\', "/");
        let rel = if base.is_empty() {
            sanitized_name.clone()
        } else {
            format!("{}/{}", base, sanitized_name)
        };

        // Validate the combined path
        let vp = ValidatedPath::new_destination(&rel)?;
        let full = vp.into_pathbuf();

        info!("persisting upload to {:?}", full);

        // Ensure the directory tree exists
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::Internal(format!("Failed to create {:?}: {}", parent, e))
            })?;
        }

        // Write the file out
        file.persist_to(&full).await.map_err(|e| {
            AppError::Internal(format!("Failed to persist {:?}: {}", full, e))
        })?;
    }

    info!("all uploads processed successfully");
    Ok(Status::Ok)
}
