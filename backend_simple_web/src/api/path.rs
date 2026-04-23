// backend_simple_web/src/api/path.rs

use std::path::{Component, Path, PathBuf};
use rocket::tokio::fs;

use super::error::AppError;
use super::{ROOT, clean};

/// A validated, safe path under ROOT. Cannot be constructed without validation.
pub struct ValidatedPath {
    full: PathBuf,
}

impl ValidatedPath {
    /// Validate a user-supplied relative path for general use.
    /// Rejects empty paths, null bytes, dangerous chars, and traversal components.
    pub fn new(rel: &str) -> Result<Self, AppError> {
        let cleaned = clean(rel);
        if cleaned.is_empty() {
            return Err(AppError::BadRequest("Path is empty".into()));
        }
        sanitize(&cleaned)?;
        let full = Path::new(ROOT).join(&cleaned);
        check_no_traversal(&full)?;
        Ok(Self { full })
    }

    /// For destinations that may not exist yet (move targets, new files).
    /// Validates the path itself but does not require it to exist on disk.
    pub fn new_destination(rel: &str) -> Result<Self, AppError> {
        let cleaned = clean(rel);
        if cleaned.is_empty() {
            return Err(AppError::BadRequest("Path is empty".into()));
        }
        sanitize(&cleaned)?;
        let full = Path::new(ROOT).join(&cleaned);
        check_no_traversal(&full)?;

        // If parent exists, verify it's under ROOT via canonicalize
        if let Some(parent) = full.parent() {
            if parent.exists() {
                let root_canon = std::path::Path::new(ROOT)
                    .canonicalize()
                    .map_err(|e| AppError::Internal(format!("Failed to resolve root: {}", e)))?;
                let parent_canon = parent
                    .canonicalize()
                    .map_err(|e| AppError::Internal(format!("Failed to resolve parent: {}", e)))?;
                if !parent_canon.starts_with(&root_canon) {
                    return Err(AppError::BadRequest("Path escapes root".into()));
                }
            }
        }
        Ok(Self { full })
    }

    /// For source paths that must already exist on disk.
    /// Uses canonicalize to resolve symlinks and verify the path is under ROOT.
    pub async fn existing(rel: &str) -> Result<Self, AppError> {
        let vp = Self::new(rel)?;
        let canon = fs::canonicalize(&vp.full)
            .await
            .map_err(|_| AppError::NotFound(format!("Path does not exist: {}", rel)))?;
        let root_canon = fs::canonicalize(ROOT)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to resolve root: {}", e)))?;
        if !canon.starts_with(&root_canon) {
            return Err(AppError::BadRequest("Path escapes root".into()));
        }
        Ok(Self { full: canon })
    }

    pub fn as_path(&self) -> &Path {
        &self.full
    }

    pub fn into_pathbuf(self) -> PathBuf {
        self.full
    }
}

fn sanitize(path: &str) -> Result<(), AppError> {
    if path.contains('\0') {
        return Err(AppError::BadRequest("Path contains null bytes".into()));
    }
    let dangerous = ['<', '>', ':', '"', '|', '?', '*'];
    if path.chars().any(|c| dangerous.contains(&c)) {
        return Err(AppError::BadRequest("Path contains invalid characters".into()));
    }
    Ok(())
}

fn check_no_traversal(path: &Path) -> Result<(), AppError> {
    if path.components().any(|c| c == Component::ParentDir) {
        return Err(AppError::BadRequest("Path traversal not allowed".into()));
    }
    Ok(())
}
