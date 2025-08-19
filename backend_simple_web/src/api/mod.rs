// backend_simple_web/src/api/mod.rs

pub mod files;
pub mod upload;
pub mod move_ops;
pub mod git;

pub const ROOT: &str = "/public_site";

// Percent‑decode helper ------------------------------------------------------
pub fn clean(rel: &str) -> String {
    let trimmed = rel.trim_start_matches('/');
    urlencoding::decode(trimmed)
        .unwrap_or_else(|_| trimmed.into())
        .into_owned()
}

// Re-export all route handlers for main.rs
pub use files::{list_files, get_file, save_file, delete_file};
pub use upload::upload as upload_files;
pub use move_ops::move_entry;
pub use git::{setup_git_repo, pull_repo};