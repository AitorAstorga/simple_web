// backend_simple_web/src/api/mod.rs

pub mod files;
pub mod upload;
pub mod move_ops;
pub mod git;
pub mod themes;

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
pub use git::{setup_git_repo, pull_repo, test_git_repo, get_auto_pull_config, set_auto_pull_config, get_git_status, commit_changes, push_repo, force_pull_repo};
pub use themes::{list_themes, get_theme, save_theme, delete_theme};