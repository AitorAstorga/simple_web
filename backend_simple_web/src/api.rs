use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::fs::{NamedFile, TempFile};
use rocket::http::Status;
use rocket::tokio::fs;
use rocket::form::Form;
use std::path::{Path, PathBuf};

use crate::auth::Admin;

const ROOT: &str = "../public_site";

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct FileEntry {
    path: String,
    is_dir: bool,
}

// Percent‑decode helper ------------------------------------------------------
fn clean(rel: &str) -> String {
    let trimmed = rel.trim_start_matches('/');
    urlencoding::decode(trimmed).unwrap_or_else(|_| trimmed.into()).into_owned()
}

// ------------- LIST FILES ---------------------------------------------------
// Example: GET /api/files               → list ROOT
//          GET /api/files?path=img/logo → list ./img/logo
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
// Example: GET /api/file?path=index.html
#[get("/file?<path>")]
pub async fn get_file(path: Option<String>, _admin: Admin) -> Option<NamedFile> {
    let rel = path.map(|p| clean(&p)).filter(|p| !p.is_empty())?;
    let full = Path::new(ROOT).join(rel);
    NamedFile::open(full).await.ok()
}

// ------------- SAVE FILE ----------------------------------------------------
// Example: POST /api/file  JSON {"path":"css/app.css","content":"body{}"}
#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct FileSave { path: String, content: String }

#[post("/file", data = "<payload>")]
pub async fn save_file(_admin: Admin, payload: Json<FileSave>) -> Result<Status, Status> {
    let rel = clean(&payload.path);
    if rel.is_empty() { return Err(Status::BadRequest); }

    let full = Path::new(ROOT).join(&rel);
    if fs::metadata(&full).await.ok().map(|m| m.is_dir()).unwrap_or(false) {
        return Err(Status::BadRequest); // target is a directory
    }

    if let Some(parent) = full.parent() { fs::create_dir_all(parent).await.ok(); }
    fs::write(&full, &payload.content).await.map_err(|_| Status::InternalServerError)?;
    Ok(Status::Ok)
}

// ------------- DELETE FILE / DIR -------------------------------------------
// Example: DELETE /api/file?path=img/logo.png
#[delete("/file?<path>")]
pub async fn delete_file(path: Option<String>, _admin: Admin) -> Result<Status, Status> {
    let Some(rel) = path.map(|p| clean(&p)).filter(|p| !p.is_empty()) else {
        return Err(Status::BadRequest);
    };
    let full = Path::new(ROOT).join(&rel);

    if fs::metadata(&full).await.ok().map(|m| m.is_file()).unwrap_or(false) {
        fs::remove_file(full).await.ok();
    } else {
        fs::remove_dir_all(full).await.ok();
    }
    Ok(Status::Ok)
}

// ------------- MOVE / RENAME -----------------------------------------------
// Example: POST /api/move  JSON {"from":"old.html","to":"new.html"}
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FileMove { from: String, to: String }

/// Reject `..` and ensure the final canonical path is still inside `ROOT`.
async fn resolve_into_root(rel: &str) -> Result<PathBuf, Status> {
    let cleaned   = clean(rel);
    if cleaned.is_empty() { return Err(Status::BadRequest); }

    let full = Path::new(ROOT).join(&cleaned);
    let canonical = fs::canonicalize(&full).await
        .map_err(|_| Status::BadRequest)?;
    let root_canon = fs::canonicalize(ROOT).await
        .map_err(|_| Status::InternalServerError)?;

    if !canonical.starts_with(&root_canon) {
        return Err(Status::BadRequest);      // path traversal attempt
    }
    Ok(canonical)
}

#[post("/move", data = "<payload>")]
pub async fn move_entry(
    _admin: Admin,
    payload: Json<FileMove>
) -> Result<Status, Status> {

    let src = resolve_into_root(&payload.from).await?;
    let dst = resolve_into_root(&payload.to  ).await?;

    // Prevent moving a directory *inside itself* (would loop forever)
    if src.is_dir() && dst.starts_with(&src) {
        return Err(Status::BadRequest);
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).await.ok();
    }

    fs::rename(&src, &dst).await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Status::Ok)
}

// ---------- UPLOAD MULTIPLE FILES / FOLDERS ---------------------------------
#[derive(FromForm)]
pub struct Upload<'r> {
    // Each selected file is sent as “files”; its filename keeps the relative path
    #[field(name = "files")]
    files: Vec<TempFile<'r>>,
}

#[post("/upload", data = "<payload>")]
pub async fn upload(
    _admin: Admin,
    mut payload: Form<Upload<'_>>,
) -> Result<Status, Status> {
    for file in &mut payload.files {
        // Browser-supplied name, e.g.  "docs/readme.md" or "images/logo.svg"
        let Some(rel) = file.name().map(|n| clean(&n.to_string())) else {
            continue;                       // ignore unnamed parts
        };

        // Reject absolute paths or traversal attempts
        if rel.is_empty() || rel.contains("..") {
            return Err(Status::BadRequest);
        }

        let full = Path::new(ROOT).join(&rel);

        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).await.ok();
        }

        // Persist the temporary upload to its final location
        file.persist_to(&full).await
            .map_err(|_| Status::InternalServerError)?;
    }

    Ok(Status::Ok)
}
