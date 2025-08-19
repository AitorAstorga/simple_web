// backend_simple_web/src/api/git.rs
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::fs;
use std::path::Path;
use git2::{Repository, Cred, FetchOptions, RemoteCallbacks};

use crate::auth::Admin;
use crate::scheduler::{get_scheduler, AutoPullConfig};
use super::ROOT;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GitRepoConfig {
    url: String,
    branch: Option<String>,
    username: Option<String>,
    token: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GitStatus {
    pub success: bool,
    pub message: String,
    pub commit_hash: Option<String>,
}

/// Setup a Git repository
/// ### Arguments:
/// - `config`: Git repository configuration (URL, branch, credentials)
/// ### Examples:
/// - POST /api/git/setup  JSON ```{"url":"https://github.com/user/repo.git","branch":"main"}```
#[post("/git/setup", data = "<config>")]
pub async fn setup_git_repo(config: Json<GitRepoConfig>, _admin: Admin) -> Json<GitStatus> {
    info!("🔧 Setting up Git repository: {}", config.url);
    
    let repo_path = Path::new(ROOT);
    
    // Check if directory exists and is not empty
    if repo_path.exists() {
        if let Ok(mut entries) = fs::read_dir(repo_path).await {
            if entries.next_entry().await.unwrap().is_some() {
                // Directory is not empty, try to open existing repo
                match Repository::open(repo_path) {
                    Ok(repo) => {
                        info!("📂 Found existing Git repository");
                        // Update remote URL if different
                        match repo.find_remote("origin") {
                            Ok(remote) => {
                                if remote.url().unwrap_or("") != config.url {
                                    // Remove and recreate the remote with new URL
                                    if let Err(e) = repo.remote_delete("origin") {
                                        error!("❌ Failed to delete remote: {}", e);
                                        return Json(GitStatus {
                                            success: false,
                                            message: format!("Failed to update remote URL: {}", e),
                                            commit_hash: None,
                                        });
                                    }
                                    if let Err(e) = repo.remote("origin", &config.url) {
                                        error!("❌ Failed to recreate remote: {}", e);
                                        return Json(GitStatus {
                                            success: false,
                                            message: format!("Failed to update remote URL: {}", e),
                                            commit_hash: None,
                                        });
                                    }
                                }
                            }
                            Err(_) => {
                                // Add origin remote
                                if let Err(e) = repo.remote("origin", &config.url) {
                                    error!("❌ Failed to add remote: {}", e);
                                    return Json(GitStatus {
                                        success: false,
                                        message: format!("Failed to add remote: {}", e),
                                        commit_hash: None,
                                    });
                                }
                            }
                        }
                        
                        let head = repo.head().ok().and_then(|h| h.target()).map(|oid| oid.to_string());
                        return Json(GitStatus {
                            success: true,
                            message: "Repository configured successfully".to_string(),
                            commit_hash: head,
                        });
                    }
                    Err(_) => {
                        // Not a git repo, need to clear directory or initialize
                        warn!("📁 Directory exists but is not a Git repository");
                        return Json(GitStatus {
                            success: false,
                            message: "Directory exists but is not a Git repository. Please clear it first.".to_string(),
                            commit_hash: None,
                        });
                    }
                }
            }
        }
    }
    
    // Clone the repository
    info!("📥 Cloning repository from {}", config.url);
    
    let mut builder = git2::build::RepoBuilder::new();
    
    // Setup credentials if provided
    if config.username.is_some() && config.token.is_some() {
        let username = config.username.as_ref().unwrap().clone();
        let token = config.token.as_ref().unwrap().clone();
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&username, &token)
        });
        
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);
    }
    
    // Set branch if specified
    if let Some(branch) = &config.branch {
        builder.branch(branch);
    }
    
    match builder.clone(&config.url, repo_path) {
        Ok(repo) => {
            info!("✅ Repository cloned successfully");
            let head = repo.head().ok().and_then(|h| h.target()).map(|oid| oid.to_string());
            Json(GitStatus {
                success: true,
                message: "Repository cloned successfully".to_string(),
                commit_hash: head,
            })
        }
        Err(e) => {
            error!("❌ Failed to clone repository: {}", e);
            Json(GitStatus {
                success: false,
                message: format!("Failed to clone repository: {}", e),
                commit_hash: None,
            })
        }
    }
}

/// Test Git repository connection without setting up
/// ### Arguments:
/// - `config`: Git repository configuration to test
/// ### Examples:
/// - POST /api/git/test  JSON ```{"url":"https://github.com/user/repo.git","branch":"main"}```
#[post("/git/test", data = "<config>")]
pub async fn test_git_repo(config: Json<GitRepoConfig>, _admin: Admin) -> Json<GitStatus> {
    info!("🧪 Testing Git repository connection: {}", config.url);
    
    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir();
    let temp_path = match temp_dir {
        Ok(ref dir) => dir.path(),
        Err(e) => {
            error!("❌ Failed to create temporary directory: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to create temporary directory: {}", e),
                commit_hash: None,
            });
        }
    };
    
    let mut builder = git2::build::RepoBuilder::new();
    
    // Setup credentials if provided
    if config.username.is_some() && config.token.is_some() {
        let username = config.username.as_ref().unwrap().clone();
        let token = config.token.as_ref().unwrap().clone();
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&username, &token)
        });
        
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);
    }
    
    // Set branch if specified
    if let Some(branch) = &config.branch {
        builder.branch(branch);
    }
    
    // Try to clone to temporary directory (this tests connection without affecting the main repo)
    match builder.clone(&config.url, temp_path) {
        Ok(repo) => {
            info!("✅ Repository connection test successful");
            let head = repo.head().ok().and_then(|h| h.target()).map(|oid| oid.to_string());
            // Temporary directory is automatically cleaned up when temp_dir is dropped
            Json(GitStatus {
                success: true,
                message: "Connection test passed - repository is accessible".to_string(),
                commit_hash: head,
            })
        }
        Err(e) => {
            error!("❌ Repository connection test failed: {}", e);
            Json(GitStatus {
                success: false,
                message: format!("Connection test failed: {}", e),
                commit_hash: None,
            })
        }
    }
}

/// Pull latest changes from the Git repository
/// ### Examples:
/// - POST /api/git/pull
#[post("/git/pull")]
pub async fn pull_repo(_admin: Admin) -> Json<GitStatus> {
    info!("📥 Pulling latest changes from repository");
    
    let repo_path = Path::new(ROOT);
    
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            error!("❌ Failed to open repository: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("No Git repository found: {}", e),
                commit_hash: None,
            });
        }
    };
    
    // Get the remote
    let mut remote = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(e) => {
            error!("❌ Failed to find remote 'origin': {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("No remote 'origin' found: {}", e),
                commit_hash: None,
            });
        }
    };
    
    // Fetch the latest changes
    match remote.fetch(&[] as &[&str], None, None) {
        Ok(_) => info!("📡 Fetched latest changes"),
        Err(e) => {
            error!("❌ Failed to fetch: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to fetch changes: {}", e),
                commit_hash: None,
            });
        }
    }
    
    // Get the current branch
    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => {
            error!("❌ Failed to get HEAD: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to get current branch: {}", e),
                commit_hash: None,
            });
        }
    };
    
    let branch_name = head.shorthand().unwrap_or("main");
    let remote_branch_name = format!("origin/{}", branch_name);
    
    // Get the remote branch
    let remote_branch = match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
        Ok(branch) => branch,
        Err(e) => {
            error!("❌ Failed to find remote branch {}: {}", remote_branch_name, e);
            return Json(GitStatus {
                success: false,
                message: format!("Remote branch '{}' not found: {}", remote_branch_name, e),
                commit_hash: None,
            });
        }
    };
    
    let remote_commit = match remote_branch.get().peel_to_commit() {
        Ok(commit) => commit,
        Err(e) => {
            error!("❌ Failed to get remote commit: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to get remote commit: {}", e),
                commit_hash: None,
            });
        }
    };
    
    // Reset to the remote commit (hard reset)
    match repo.reset(&remote_commit.as_object(), git2::ResetType::Hard, None) {
        Ok(_) => {
            info!("✅ Successfully pulled latest changes");
            Json(GitStatus {
                success: true,
                message: "Successfully pulled latest changes".to_string(),
                commit_hash: Some(remote_commit.id().to_string()),
            })
        }
        Err(e) => {
            error!("❌ Failed to reset to remote commit: {}", e);
            Json(GitStatus {
                success: false,
                message: format!("Failed to update to latest changes: {}", e),
                commit_hash: None,
            })
        }
    }
}

/// Internal pull function for scheduled operations (no auth required)
pub async fn pull_repo_internal() -> Result<GitStatus, String> {
    info!("📥 Internal pull operation started");
    
    let repo_path = Path::new(ROOT);
    
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            error!("❌ Failed to open repository: {}", e);
            return Err(format!("No Git repository found: {}", e));
        }
    };
    
    // Get the remote
    let mut remote = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(e) => {
            error!("❌ Failed to find remote 'origin': {}", e);
            return Err(format!("No remote 'origin' found: {}", e));
        }
    };
    
    // Fetch the latest changes
    match remote.fetch(&[] as &[&str], None, None) {
        Ok(_) => info!("📡 Fetched latest changes"),
        Err(e) => {
            error!("❌ Failed to fetch: {}", e);
            return Err(format!("Failed to fetch changes: {}", e));
        }
    }
    
    // Get the current branch
    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => {
            error!("❌ Failed to get HEAD: {}", e);
            return Err(format!("Failed to get current branch: {}", e));
        }
    };
    
    let branch_name = head.shorthand().unwrap_or("main");
    let remote_branch_name = format!("origin/{}", branch_name);
    
    // Get the remote branch
    let remote_branch = match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
        Ok(branch) => branch,
        Err(e) => {
            error!("❌ Failed to find remote branch {}: {}", remote_branch_name, e);
            return Err(format!("Remote branch '{}' not found: {}", remote_branch_name, e));
        }
    };
    
    let remote_commit = match remote_branch.get().peel_to_commit() {
        Ok(commit) => commit,
        Err(e) => {
            error!("❌ Failed to get remote commit: {}", e);
            return Err(format!("Failed to get remote commit: {}", e));
        }
    };
    
    // Reset to the remote commit (hard reset)
    match repo.reset(&remote_commit.as_object(), git2::ResetType::Hard, None) {
        Ok(_) => {
            info!("✅ Internal pull completed successfully");
            Ok(GitStatus {
                success: true,
                message: "Successfully pulled latest changes".to_string(),
                commit_hash: Some(remote_commit.id().to_string()),
            })
        }
        Err(e) => {
            error!("❌ Failed to reset to remote commit: {}", e);
            Err(format!("Failed to update to latest changes: {}", e))
        }
    }
}

/// Get auto-pull configuration
/// ### Examples:
/// - GET /api/git/auto-pull
#[get("/git/auto-pull")]
pub async fn get_auto_pull_config(_admin: Admin) -> Json<AutoPullConfig> {
    let scheduler = get_scheduler().await;
    Json(scheduler.get_config().await)
}

/// Set auto-pull configuration
/// ### Arguments:
/// - `config`: Auto-pull configuration
/// ### Examples:
/// - POST /api/git/auto-pull  JSON ```{"enabled":true,"interval_minutes":30}```
#[post("/git/auto-pull", data = "<config>")]
pub async fn set_auto_pull_config(config: Json<AutoPullConfig>, _admin: Admin) -> Json<GitStatus> {
    let scheduler = get_scheduler().await;
    
    match scheduler.update_config(config.into_inner()).await {
        Ok(_) => {
            info!("✅ Auto-pull configuration updated successfully");
            Json(GitStatus {
                success: true,
                message: "Auto-pull configuration updated successfully".to_string(),
                commit_hash: None,
            })
        }
        Err(e) => {
            error!("❌ Failed to update auto-pull configuration: {}", e);
            Json(GitStatus {
                success: false,
                message: format!("Failed to update auto-pull configuration: {}", e),
                commit_hash: None,
            })
        }
    }
}