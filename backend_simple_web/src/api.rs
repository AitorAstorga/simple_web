// backend_simple_web/src/api.rs
use rocket::form::Form;
use rocket::fs::{NamedFile, TempFile};
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::fs;
use std::path::{Component, Path, PathBuf};

use crate::auth::Admin;

const ROOT: &str = "/public_site";

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FileEntry {
    path: String,
    is_dir: bool,
}

// Percent‑decode helper ------------------------------------------------------
fn clean(rel: &str) -> String {
    let trimmed = rel.trim_start_matches('/');
    urlencoding::decode(trimmed)
        .unwrap_or_else(|_| trimmed.into())
        .into_owned()
}

// ------------- LIST FILES ---------------------------------------------------
/// List files in a directory
/// ### Arguments:
/// - `path` (optional): relative path inside the public site
/// ### Examples:
/// - GET /api/files               → list ROOT
/// - GET /api/files?path=img/logo → list ./img/logo
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
#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct FileBody {
    content: String,
}

/// Save a file
/// ### Arguments:
/// - `path` (optional): relative path inside the public site
/// - `content` (required): the file content
/// ### Examples:
/// - POST /api/file  JSON ```{"path":"css/app.css","content":"body{}"}```
#[post("/file?<path>", data = "<body>")]
pub async fn save_file(_admin: Admin, path: &str, body: Json<FileBody>) -> Result<Status, Status> {
    let rel = clean(path);
    if rel.is_empty() {
        return Err(Status::BadRequest);
    }

    let full = Path::new(ROOT).join(&rel);
    if fs::metadata(&full)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        return Err(Status::BadRequest); // target is a directory
    }

    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|_| Status::InternalServerError)?;
    }
    fs::write(&full, &body.content)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Status::Ok)
}

// ------------- DELETE FILE / DIR -------------------------------------------
/// Delete a file or directory
/// ### Arguments:
/// - `path` (optional): relative path inside the public site
/// ### Examples:
/// - DELETE /api/file?path=img/logo.png
#[delete("/file?<path>")]
pub async fn delete_file(path: Option<String>, _admin: Admin) -> Result<Status, Status> {
    let Some(rel) = path.map(|p| clean(&p)).filter(|p| !p.is_empty()) else {
        return Err(Status::BadRequest);
    };
    let full = Path::new(ROOT).join(&rel);

    if fs::metadata(&full)
        .await
        .ok()
        .map(|m| m.is_file())
        .unwrap_or(false)
    {
        fs::remove_file(full).await.ok();
    } else {
        fs::remove_dir_all(full).await.ok();
    }
    Ok(Status::Ok)
}

// ------------- MOVE / RENAME -----------------------------------------------
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

// ---------- UPLOAD MULTIPLE FILES / FOLDERS ---------------------------------
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

        // Validate: no absolute, no “..”
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
