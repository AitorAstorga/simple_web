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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GitFileStatus {
    pub path: String,
    pub status: String, // "new", "modified", "deleted", "renamed", "untracked"
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
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
    
    // Create a temporary repository in memory to test remote access
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            error!("❌ Failed to create temporary directory: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to create temporary directory: {}", e),
                commit_hash: None,
            });
        }
    };
    
    // Initialize a temporary repository
    let repo = match Repository::init(temp_dir.path()) {
        Ok(repo) => repo,
        Err(e) => {
            error!("❌ Failed to initialize temporary repository: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to initialize test repository: {}", e),
                commit_hash: None,
            });
        }
    };
    
    // Add the remote
    let mut remote = match repo.remote("origin", &config.url) {
        Ok(remote) => remote,
        Err(e) => {
            error!("❌ Failed to add remote: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Invalid repository URL: {}", e),
                commit_hash: None,
            });
        }
    };
    
    // Setup credentials if provided
    let mut callbacks = RemoteCallbacks::new();
    if config.username.is_some() && config.token.is_some() {
        let username = config.username.as_ref().unwrap().clone();
        let token = config.token.as_ref().unwrap().clone();
        
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&username, &token)
        });
    }
    
    // Just connect and list remote refs (much faster than cloning)
    let result = remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None);
    match result {
        Ok(connection) => {
            // Immediately disconnect to clean up resources
            drop(connection);
            info!("✅ Repository connection test successful");
            Json(GitStatus {
                success: true,
                message: "Connection test passed - repository is accessible".to_string(),
                commit_hash: None,
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

/// Pull latest changes from the Git repository (non-destructive merge)
/// ### Examples:
/// - POST /api/git/pull
#[post("/git/pull")]
pub async fn pull_repo(_admin: Admin) -> Json<GitStatus> {
    info!("📥 Pulling latest changes from repository (merge)");
    
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
    
    // Check if there are uncommitted changes
    match repo.statuses(None) {
        Ok(statuses) => {
            if !statuses.is_empty() {
                return Json(GitStatus {
                    success: false,
                    message: "Cannot pull with uncommitted changes. Please commit or stash changes first.".to_string(),
                    commit_hash: None,
                });
            }
        }
        Err(e) => {
            error!("❌ Failed to check repository status: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to check repository status: {}", e),
                commit_hash: None,
            });
        }
    }
    
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
    
    let local_commit = match head.target() {
        Some(oid) => match repo.find_commit(oid) {
            Ok(commit) => commit,
            Err(e) => {
                error!("❌ Failed to find local commit: {}", e);
                return Json(GitStatus {
                    success: false,
                    message: format!("Failed to find local commit: {}", e),
                    commit_hash: None,
                });
            }
        },
        None => {
            error!("❌ HEAD has no target");
            return Json(GitStatus {
                success: false,
                message: "HEAD has no target".to_string(),
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
    
    // Check if we're already up to date
    if local_commit.id() == remote_commit.id() {
        info!("✅ Already up to date");
        return Json(GitStatus {
            success: true,
            message: "Already up to date".to_string(),
            commit_hash: Some(local_commit.id().to_string()),
        });
    }
    
    // Check if we can fast-forward
    let (ahead, behind) = match repo.graph_ahead_behind(local_commit.id(), remote_commit.id()) {
        Ok(counts) => counts,
        Err(e) => {
            error!("❌ Failed to calculate ahead/behind: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to calculate repository status: {}", e),
                commit_hash: None,
            });
        }
    };
    
    if ahead == 0 {
        // We can fast-forward
        let local_branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
            Ok(branch) => branch,
            Err(e) => {
                error!("❌ Failed to find local branch: {}", e);
                return Json(GitStatus {
                    success: false,
                    message: format!("Failed to find local branch: {}", e),
                    commit_hash: None,
                });
            }
        };
        
        // Update the branch reference
        match local_branch.into_reference().set_target(remote_commit.id(), "Fast-forward pull") {
            Ok(_) => {
                // Update HEAD
                if let Err(e) = repo.set_head(&format!("refs/heads/{}", branch_name)) {
                    error!("❌ Failed to update HEAD: {}", e);
                    return Json(GitStatus {
                        success: false,
                        message: format!("Failed to update HEAD: {}", e),
                        commit_hash: None,
                    });
                }
                
                // Update working directory
                if let Err(e) = repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force())) {
                    error!("❌ Failed to update working directory: {}", e);
                    return Json(GitStatus {
                        success: false,
                        message: format!("Failed to update working directory: {}", e),
                        commit_hash: None,
                    });
                }
                
                info!("✅ Successfully fast-forwarded to latest changes");
                Json(GitStatus {
                    success: true,
                    message: format!("Successfully pulled {} new commits", behind),
                    commit_hash: Some(remote_commit.id().to_string()),
                })
            }
            Err(e) => {
                error!("❌ Failed to update branch: {}", e);
                Json(GitStatus {
                    success: false,
                    message: format!("Failed to update branch: {}", e),
                    commit_hash: None,
                })
            }
        }
    } else {
        // We have local commits, need to merge or rebase
        Json(GitStatus {
            success: false,
            message: format!("Cannot pull: you have {} unpushed commits. Please push your changes first, or use Force Pull to discard local changes.", ahead),
            commit_hash: None,
        })
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

/// Get git repository status
/// ### Examples:
/// - GET /api/git/status
#[get("/git/status")]
pub async fn get_git_status(_admin: Admin) -> Json<GitRepoStatus> {
    info!("📊 Getting git repository status");
    
    let repo_path = Path::new(ROOT);
    
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) => {
            error!("❌ Failed to open repository: {}", e);
            return Json(GitRepoStatus {
                success: false,
                message: format!("No Git repository found: {}", e),
                current_branch: None,
                current_commit: None,
                remote_commit: None,
                behind_count: 0,
                ahead_count: 0,
                has_changes: false,
                has_staged_changes: false,
                has_unstaged_changes: false,
                changed_files: vec![],
                untracked_files: vec![],
            });
        }
    };

    // Get current branch
    let current_branch = repo.head().ok()
        .and_then(|head| head.shorthand().map(|s| s.to_string()));

    // Get current commit
    let current_commit = repo.head().ok()
        .and_then(|head| head.target())
        .map(|oid| oid.to_string());

    // Get remote commit
    let default_branch = "main".to_string();
    let branch_name = current_branch.as_ref().unwrap_or(&default_branch);
    let remote_branch_name = format!("origin/{}", branch_name);
    
    let remote_commit = repo.find_branch(&remote_branch_name, git2::BranchType::Remote).ok()
        .and_then(|branch| branch.get().peel_to_commit().ok())
        .map(|commit| commit.id().to_string());

    // Count commits behind/ahead
    let (behind_count, ahead_count) = if let (Some(local), Some(remote)) = (
        repo.head().ok().and_then(|h| h.target()),
        repo.find_branch(&remote_branch_name, git2::BranchType::Remote).ok()
            .and_then(|b| b.get().target())
    ) {
        let ahead = repo.graph_ahead_behind(local, remote).unwrap_or((0, 0));
        (ahead.1, ahead.0)
    } else {
        (0, 0)
    };

    // Get working directory status
    let mut changed_files = Vec::new();
    let mut untracked_files = Vec::new();
    let mut has_staged_changes = false;
    let mut has_unstaged_changes = false;

    match repo.statuses(None) {
        Ok(statuses) => {
            for entry in statuses.iter() {
                let path = entry.path().unwrap_or("").to_string();
                let flags = entry.status();

                if flags.contains(git2::Status::INDEX_NEW) {
                    changed_files.push(GitFileStatus {
                        path: path.clone(),
                        status: "staged_new".to_string(),
                    });
                    has_staged_changes = true;
                } else if flags.contains(git2::Status::INDEX_MODIFIED) {
                    changed_files.push(GitFileStatus {
                        path: path.clone(),
                        status: "staged_modified".to_string(),
                    });
                    has_staged_changes = true;
                } else if flags.contains(git2::Status::INDEX_DELETED) {
                    changed_files.push(GitFileStatus {
                        path: path.clone(),
                        status: "staged_deleted".to_string(),
                    });
                    has_staged_changes = true;
                } else if flags.contains(git2::Status::INDEX_RENAMED) {
                    changed_files.push(GitFileStatus {
                        path: path.clone(),
                        status: "staged_renamed".to_string(),
                    });
                    has_staged_changes = true;
                }

                if flags.contains(git2::Status::WT_NEW) {
                    untracked_files.push(path.clone());
                    has_unstaged_changes = true;
                } else if flags.contains(git2::Status::WT_MODIFIED) {
                    changed_files.push(GitFileStatus {
                        path: path.clone(),
                        status: "modified".to_string(),
                    });
                    has_unstaged_changes = true;
                } else if flags.contains(git2::Status::WT_DELETED) {
                    changed_files.push(GitFileStatus {
                        path: path.clone(),
                        status: "deleted".to_string(),
                    });
                    has_unstaged_changes = true;
                } else if flags.contains(git2::Status::WT_RENAMED) {
                    changed_files.push(GitFileStatus {
                        path: path.clone(),
                        status: "renamed".to_string(),
                    });
                    has_unstaged_changes = true;
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get repository status: {}", e);
            return Json(GitRepoStatus {
                success: false,
                message: format!("Failed to get repository status: {}", e),
                current_branch,
                current_commit,
                remote_commit,
                behind_count: 0,
                ahead_count: 0,
                has_changes: false,
                has_staged_changes: false,
                has_unstaged_changes: false,
                changed_files: vec![],
                untracked_files: vec![],
            });
        }
    }

    let has_changes = has_staged_changes || has_unstaged_changes || !untracked_files.is_empty();

    Json(GitRepoStatus {
        success: true,
        message: "Repository status retrieved successfully".to_string(),
        current_branch,
        current_commit,
        remote_commit,
        behind_count,
        ahead_count,
        has_changes,
        has_staged_changes,
        has_unstaged_changes,
        changed_files,
        untracked_files,
    })
}

/// Commit pending changes
/// ### Arguments:
/// - `message`: Commit message
/// ### Examples:
/// - POST /api/git/commit  JSON ```{"message":"Updated files via simple_web"}```
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CommitRequest {
    message: String,
}

#[post("/git/commit", data = "<request>")]
pub async fn commit_changes(request: Json<CommitRequest>, _admin: Admin) -> Json<GitStatus> {
    info!("📝 Committing changes with message: {}", request.message);
    
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

    // Add all changes to staging area
    let mut index = match repo.index() {
        Ok(index) => index,
        Err(e) => {
            error!("❌ Failed to get repository index: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to access repository index: {}", e),
                commit_hash: None,
            });
        }
    };

    // Add all changes (including deletions)
    if let Err(e) = index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None) {
        error!("❌ Failed to add changes to index: {}", e);
        return Json(GitStatus {
            success: false,
            message: format!("Failed to stage changes: {}", e),
            commit_hash: None,
        });
    }

    // Update index
    if let Err(e) = index.update_all(["*"].iter(), None) {
        error!("❌ Failed to update index: {}", e);
        return Json(GitStatus {
            success: false,
            message: format!("Failed to update index: {}", e),
            commit_hash: None,
        });
    }

    if let Err(e) = index.write() {
        error!("❌ Failed to write index: {}", e);
        return Json(GitStatus {
            success: false,
            message: format!("Failed to write index: {}", e),
            commit_hash: None,
        });
    }

    // Check if there are any changes to commit
    let tree_id = match index.write_tree() {
        Ok(id) => id,
        Err(e) => {
            error!("❌ Failed to write tree: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to create commit tree: {}", e),
                commit_hash: None,
            });
        }
    };

    let tree = match repo.find_tree(tree_id) {
        Ok(tree) => tree,
        Err(e) => {
            error!("❌ Failed to find tree: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to find commit tree: {}", e),
                commit_hash: None,
            });
        }
    };

    // Get HEAD commit (parent)
    let parent_commit = repo.head().ok()
        .and_then(|head| head.target())
        .and_then(|oid| repo.find_commit(oid).ok());

    // Create commit signature
    let signature = git2::Signature::now("Simple Web", "noreply@simple-web.local")
        .unwrap_or_else(|_| git2::Signature::now("Unknown", "unknown@local").unwrap());

    // Create the commit
    let parents = if let Some(ref parent) = parent_commit {
        vec![parent]
    } else {
        vec![]
    };

    let commit_oid = match repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &request.message,
        &tree,
        &parents,
    ) {
        Ok(oid) => oid,
        Err(e) => {
            error!("❌ Failed to create commit: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to create commit: {}", e),
                commit_hash: None,
            });
        }
    };

    info!("✅ Commit created successfully: {}", commit_oid);
    Json(GitStatus {
        success: true,
        message: format!("Changes committed successfully"),
        commit_hash: Some(commit_oid.to_string()),
    })
}

/// Push local commits to the remote repository
/// ### Examples:
/// - POST /api/git/push
#[post("/git/push")]
pub async fn push_repo(_admin: Admin) -> Json<GitStatus> {
    info!("📤 Pushing local commits to remote repository");
    
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
    
    // Check if there are uncommitted changes
    match repo.statuses(None) {
        Ok(statuses) => {
            if !statuses.is_empty() {
                return Json(GitStatus {
                    success: false,
                    message: "Cannot push with uncommitted changes. Please commit changes first.".to_string(),
                    commit_hash: None,
                });
            }
        }
        Err(e) => {
            error!("❌ Failed to check repository status: {}", e);
            return Json(GitStatus {
                success: false,
                message: format!("Failed to check repository status: {}", e),
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
    
    // Push to remote
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    match remote.push(&[&refspec], None) {
        Ok(_) => {
            info!("✅ Successfully pushed commits to remote");
            let current_commit = head.target().map(|oid| oid.to_string());
            Json(GitStatus {
                success: true,
                message: "Successfully pushed commits to remote repository".to_string(),
                commit_hash: current_commit,
            })
        }
        Err(e) => {
            error!("❌ Failed to push to remote: {}", e);
            Json(GitStatus {
                success: false,
                message: format!("Failed to push to remote: {}. Check if you have push permissions.", e),
                commit_hash: None,
            })
        }
    }
}

/// Force pull with warning - overwrites local changes
/// ### Examples:
/// - POST /api/git/force-pull
#[post("/git/force-pull")]
pub async fn force_pull_repo(_admin: Admin) -> Json<GitStatus> {
    info!("⚠️ Force pulling - this will overwrite local changes");
    
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
            info!("✅ Successfully force pulled latest changes");
            Json(GitStatus {
                success: true,
                message: "Successfully force pulled - local changes discarded".to_string(),
                commit_hash: Some(remote_commit.id().to_string()),
            })
        }
        Err(e) => {
            error!("❌ Failed to reset to remote commit: {}", e);
            Json(GitStatus {
                success: false,
                message: format!("Failed to force pull: {}", e),
                commit_hash: None,
            })
        }
    }
}