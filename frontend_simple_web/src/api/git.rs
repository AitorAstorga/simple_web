// frontend_simple_web/src/api/git.rs
use serde::{Deserialize, Serialize};

use super::client::{self, Method};

#[derive(Serialize, Clone)]
pub struct GitRepoConfig {
    pub url: String,
    pub branch: Option<String>,
    pub username: Option<String>,
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct GitStatus {
    pub success: bool,
    pub message: String,
    pub commit_hash: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String,
}

#[derive(Deserialize, Clone)]
pub struct GitRepoStatus {
    pub success: bool,
    pub message: String,
    pub current_branch: Option<String>,
    pub current_commit: Option<String>,
    pub remote_commit: Option<String>,
    pub behind_count: usize,
    pub ahead_count: usize,
    pub has_changes: bool,
    pub has_staged_changes: bool,
    pub has_unstaged_changes: bool,
    pub changed_files: Vec<GitFileStatus>,
    pub untracked_files: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutoPullConfig {
    pub enabled: bool,
    pub interval_minutes: u32,
}

#[derive(Serialize)]
struct CommitRequest {
    message: String,
}

fn serialize_body(value: &impl Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("Failed to serialize: {}", e))
}

pub fn api_git_setup(config: GitRepoConfig, callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    match serialize_body(&config) {
        Ok(body) => client::spawn_request(Method::Post, "/api/git/setup".into(), Some(body), callback),
        Err(e) => { if let Some(cb) = callback { cb(Err(e)); } }
    }
}

pub fn api_git_test(config: GitRepoConfig, callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    match serialize_body(&config) {
        Ok(body) => client::spawn_request(Method::Post, "/api/git/test".into(), Some(body), callback),
        Err(e) => { if let Some(cb) = callback { cb(Err(e)); } }
    }
}

pub fn api_git_pull(callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    client::spawn_request(Method::Post, "/api/git/pull".into(), None, callback);
}

pub fn api_get_git_status(callback: Option<impl Fn(Result<GitRepoStatus, String>) + 'static>) {
    client::spawn_request(Method::Get, "/api/git/status".into(), None, callback);
}

pub fn api_commit_changes(message: String, callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    match serialize_body(&CommitRequest { message }) {
        Ok(body) => client::spawn_request(Method::Post, "/api/git/commit".into(), Some(body), callback),
        Err(e) => { if let Some(cb) = callback { cb(Err(e)); } }
    }
}

pub fn api_push_changes(callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    client::spawn_request(Method::Post, "/api/git/push".into(), None, callback);
}

pub fn api_force_pull(callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    client::spawn_request(Method::Post, "/api/git/force-pull".into(), None, callback);
}

pub fn api_get_auto_pull_config(callback: Option<impl Fn(Result<AutoPullConfig, String>) + 'static>) {
    client::spawn_request(Method::Get, "/api/git/auto-pull".into(), None, callback);
}

pub fn api_set_auto_pull_config(config: AutoPullConfig, callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    match serialize_body(&config) {
        Ok(body) => client::spawn_request(Method::Post, "/api/git/auto-pull".into(), Some(body), callback),
        Err(e) => { if let Some(cb) = callback { cb(Err(e)); } }
    }
}
