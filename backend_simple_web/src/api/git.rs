// backend_simple_web/src/api/git.rs
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::fs;
use std::path::Path;
use git2::{Repository, Cred, FetchOptions, PushOptions, RemoteCallbacks};

use prisma_auth::backend::AuthGuard as Admin;
use crate::scheduler::{get_scheduler, AutoPullConfig};
use super::error::AppError;
use super::ROOT;

const GIT_CREDENTIALS_PATH: &str = "/app/data/git_credentials.json";

// --- Credentials ---

#[derive(Deserialize, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
struct GitCredentials {
    username: String,
    token: String,
}

async fn save_git_credentials(username: &str, token: &str) {
    let creds = GitCredentials {
        username: username.to_string(),
        token: token.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&creds) {
        if let Err(e) = fs::write(GIT_CREDENTIALS_PATH, json).await {
            error!("Failed to save git credentials: {}", e);
        }
    }
}

async fn load_git_credentials() -> Option<GitCredentials> {
    fs::read_to_string(GIT_CREDENTIALS_PATH)
        .await
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
}

fn make_fetch_options(creds: &GitCredentials) -> FetchOptions<'_> {
    let mut opts = FetchOptions::new();
    opts.remote_callbacks(make_callbacks(creds));
    opts
}

fn make_push_options(creds: &GitCredentials) -> PushOptions<'_> {
    let mut opts = PushOptions::new();
    opts.remote_callbacks(make_callbacks(creds));
    opts
}

fn make_callbacks(creds: &GitCredentials) -> RemoteCallbacks<'_> {
    let username = creds.username.clone();
    let token = creds.token.clone();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(&username, &token)
    });
    callbacks
}

// --- Shared helpers ---

fn open_repo() -> Result<Repository, AppError> {
    Repository::open(Path::new(ROOT))
        .map_err(|e| AppError::Internal(format!("No Git repository found: {}", e)))
}

fn fetch_origin(repo: &Repository, creds: &Option<GitCredentials>) -> Result<(), AppError> {
    let mut remote = repo.find_remote("origin")
        .map_err(|e| AppError::Internal(format!("No remote 'origin' found: {}", e)))?;
    let mut fetch_opts = creds.as_ref().map(make_fetch_options);
    remote.fetch(&[] as &[&str], fetch_opts.as_mut(), None)
        .map_err(|e| AppError::Internal(format!("Failed to fetch changes: {}", e)))
}

fn current_branch_name(repo: &Repository) -> Result<String, AppError> {
    let head = repo.head()
        .map_err(|e| AppError::Internal(format!("Failed to get current branch: {}", e)))?;
    Ok(head.shorthand().unwrap_or("main").to_string())
}

fn remote_commit<'a>(repo: &'a Repository, branch: &str) -> Result<git2::Commit<'a>, AppError> {
    let remote_name = format!("origin/{}", branch);
    let remote_branch = repo.find_branch(&remote_name, git2::BranchType::Remote)
        .map_err(|e| AppError::Internal(format!("Remote branch '{}' not found: {}", remote_name, e)))?;
    remote_branch.get().peel_to_commit()
        .map_err(|e| AppError::Internal(format!("Failed to get remote commit: {}", e)))
}

fn ensure_clean_workdir(repo: &Repository, action: &str) -> Result<(), AppError> {
    let statuses = repo.statuses(None)
        .map_err(|e| AppError::Internal(format!("Failed to check repository status: {}", e)))?;
    if !statuses.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Cannot {} with uncommitted changes. Please commit changes first.", action
        )));
    }
    Ok(())
}

fn head_commit_hash(repo: &Repository) -> Option<String> {
    repo.head().ok().and_then(|h| h.target()).map(|oid| oid.to_string())
}

/// Fetch from origin and hard-reset to the remote branch tip.
fn fetch_and_reset(repo: &Repository, creds: &Option<GitCredentials>) -> Result<GitStatus, AppError> {
    fetch_origin(repo, creds)?;
    let branch = current_branch_name(repo)?;
    let commit = remote_commit(repo, &branch)?;
    repo.reset(&commit.as_object(), git2::ResetType::Hard, None)
        .map_err(|e| AppError::Internal(format!("Failed to reset to remote: {}", e)))?;
    Ok(GitStatus::ok(
        "Successfully pulled latest changes",
        Some(commit.id().to_string()),
    ))
}

// --- API types ---

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct GitRepoConfig {
    url: String,
    branch: Option<String>,
    username: Option<String>,
    token: Option<String>,
}

impl GitRepoConfig {
    async fn save_creds_if_present(&self) {
        if let (Some(username), Some(token)) = (&self.username, &self.token) {
            save_git_credentials(username, token).await;
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GitStatus {
    pub success: bool,
    pub message: String,
    pub commit_hash: Option<String>,
}

impl GitStatus {
    fn ok(message: impl Into<String>, commit_hash: Option<String>) -> Self {
        Self { success: true, message: message.into(), commit_hash }
    }
}

fn git_result(result: Result<GitStatus, AppError>) -> Json<GitStatus> {
    match result {
        Ok(status) => Json(status),
        Err(e) => {
            error!("{}", e);
            Json(GitStatus { success: false, message: e.to_string(), commit_hash: None })
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GitFileStatus {
    pub path: String,
    pub status: String,
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

impl GitRepoStatus {
    fn err(message: impl Into<String>) -> Self {
        Self {
            success: false, message: message.into(),
            current_branch: None, current_commit: None, remote_commit: None,
            behind_count: 0, ahead_count: 0,
            has_changes: false, has_staged_changes: false, has_unstaged_changes: false,
            changed_files: vec![], untracked_files: vec![],
        }
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CommitRequest {
    message: String,
}

// --- Route handlers ---

/// POST /api/git/setup
#[post("/git/setup", data = "<config>")]
pub async fn setup_git_repo(config: Json<GitRepoConfig>, _admin: Admin) -> Json<GitStatus> {
    info!("Setting up Git repository: {}", config.url);
    git_result(setup_git_repo_inner(&config).await)
}

async fn setup_git_repo_inner(config: &GitRepoConfig) -> Result<GitStatus, AppError> {
    let repo_path = Path::new(ROOT);

    // Save credentials before any git2 operations (git2 types aren't Send)
    config.save_creds_if_present().await;

    if repo_path.exists() {
        if let Ok(mut entries) = fs::read_dir(repo_path).await {
            if entries.next_entry().await
                .map_err(|e| AppError::Internal(format!("Failed to read directory: {}", e)))?
                .is_some()
            {
                return match Repository::open(repo_path) {
                    Ok(repo) => {
                        configure_existing_remote(&repo, &config.url)?;
                        Ok(GitStatus::ok("Repository configured successfully", head_commit_hash(&repo)))
                    }
                    Err(_) => Err(AppError::BadRequest(
                        "Directory exists but is not a Git repository. Please clear it first.".into()
                    )),
                };
            }
        }
    }

    clone_repo(config, repo_path)
}

fn configure_existing_remote(repo: &Repository, url: &str) -> Result<(), AppError> {
    match repo.find_remote("origin") {
        Ok(remote) => {
            if remote.url().unwrap_or("") != url {
                repo.remote_delete("origin")
                    .map_err(|e| AppError::Internal(format!("Failed to update remote URL: {}", e)))?;
                repo.remote("origin", url)
                    .map_err(|e| AppError::Internal(format!("Failed to update remote URL: {}", e)))?;
            }
        }
        Err(_) => {
            repo.remote("origin", url)
                .map_err(|e| AppError::Internal(format!("Failed to add remote: {}", e)))?;
        }
    }
    Ok(())
}

fn clone_repo(config: &GitRepoConfig, repo_path: &Path) -> Result<GitStatus, AppError> {
    info!("Cloning repository from {}", config.url);

    // Credentials should already be saved by caller before entering sync git2 code
    let creds = if let (Some(username), Some(token)) = (&config.username, &config.token) {
        Some(GitCredentials { username: username.clone(), token: token.clone() })
    } else {
        None
    };

    let mut builder = git2::build::RepoBuilder::new();
    if let Some(ref creds) = creds {
        builder.fetch_options(make_fetch_options(creds));
    }
    if let Some(branch) = &config.branch {
        builder.branch(branch);
    }

    let repo = builder.clone(&config.url, repo_path)
        .map_err(|e| AppError::Internal(format!("Failed to clone repository: {}", e)))?;
    Ok(GitStatus::ok("Repository cloned successfully", head_commit_hash(&repo)))
}

/// POST /api/git/test
#[post("/git/test", data = "<config>")]
pub async fn test_git_repo(config: Json<GitRepoConfig>, _admin: Admin) -> Json<GitStatus> {
    info!("Testing Git repository connection: {}", config.url);
    git_result(test_git_repo_inner(&config).await)
}

async fn test_git_repo_inner(config: &GitRepoConfig) -> Result<GitStatus, AppError> {
    // Save credentials before any git2 operations (git2 types aren't Send)
    if let (Some(username), Some(token)) = (&config.username, &config.token) {
        save_git_credentials(username, token).await;
    }

    // All git2 operations below are sync — no .await after this point
    let temp_dir = tempfile::tempdir()
        .map_err(|e| AppError::Internal(format!("Failed to create temporary directory: {}", e)))?;
    let repo = Repository::init(temp_dir.path())
        .map_err(|e| AppError::Internal(format!("Failed to initialize test repository: {}", e)))?;
    let mut remote = repo.remote("origin", &config.url)
        .map_err(|e| AppError::BadRequest(format!("Invalid repository URL: {}", e)))?;

    let mut callbacks = RemoteCallbacks::new();
    if let (Some(username), Some(token)) = (&config.username, &config.token) {
        let username = username.clone();
        let token = token.clone();
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&username, &token)
        });
    }

    remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)
        .map_err(|e| AppError::Internal(format!("Connection test failed: {}", e)))?;

    Ok(GitStatus::ok("Connection test passed - repository is accessible", None))
}

/// POST /api/git/pull
#[post("/git/pull")]
pub async fn pull_repo(_admin: Admin) -> Json<GitStatus> {
    info!("Pulling latest changes from repository");
    git_result(pull_repo_inner().await)
}

async fn pull_repo_inner() -> Result<GitStatus, AppError> {
    // Load credentials before any git2 operations (git2 types aren't Send)
    let creds = load_git_credentials().await;

    let repo = open_repo()?;
    ensure_clean_workdir(&repo, "pull")?;
    fetch_origin(&repo, &creds)?;

    let branch_name = current_branch_name(&repo)?;
    let head = repo.head().map_err(|e| AppError::Internal(format!("Failed to get HEAD: {}", e)))?;
    let local_oid = head.target()
        .ok_or_else(|| AppError::Internal("HEAD has no target".into()))?;
    let local_commit = repo.find_commit(local_oid)
        .map_err(|e| AppError::Internal(format!("Failed to find local commit: {}", e)))?;
    let remote = remote_commit(&repo, &branch_name)?;

    if local_commit.id() == remote.id() {
        return Ok(GitStatus::ok("Already up to date", Some(local_commit.id().to_string())));
    }

    let (ahead, behind) = repo.graph_ahead_behind(local_commit.id(), remote.id())
        .map_err(|e| AppError::Internal(format!("Failed to calculate repository status: {}", e)))?;

    if ahead > 0 {
        return Err(AppError::BadRequest(format!(
            "Cannot pull: you have {} unpushed commits. Please push your changes first, or use Force Pull to discard local changes.",
            ahead
        )));
    }

    // Fast-forward
    let local_branch = repo.find_branch(&branch_name, git2::BranchType::Local)
        .map_err(|e| AppError::Internal(format!("Failed to find local branch: {}", e)))?;
    local_branch.into_reference().set_target(remote.id(), "Fast-forward pull")
        .map_err(|e| AppError::Internal(format!("Failed to update branch: {}", e)))?;
    repo.set_head(&format!("refs/heads/{}", branch_name))
        .map_err(|e| AppError::Internal(format!("Failed to update HEAD: {}", e)))?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
        .map_err(|e| AppError::Internal(format!("Failed to update working directory: {}", e)))?;

    Ok(GitStatus::ok(
        format!("Successfully pulled {} new commits", behind),
        Some(remote.id().to_string()),
    ))
}

/// Internal pull for scheduled operations (no auth guard)
pub async fn pull_repo_internal() -> Result<GitStatus, String> {
    info!("Internal pull operation started");
    // Load credentials before any git2 operations (git2 types aren't Send)
    let creds = load_git_credentials().await;
    let repo = open_repo().map_err(|e| e.to_string())?;
    fetch_and_reset(&repo, &creds).map_err(|e| e.to_string())
}

/// GET /api/git/auto-pull
#[get("/git/auto-pull")]
pub async fn get_auto_pull_config(_admin: Admin) -> Json<AutoPullConfig> {
    let scheduler = get_scheduler().await;
    Json(scheduler.get_config().await)
}

/// POST /api/git/auto-pull
#[post("/git/auto-pull", data = "<config>")]
pub async fn set_auto_pull_config(config: Json<AutoPullConfig>, _admin: Admin) -> Json<GitStatus> {
    let scheduler = get_scheduler().await;
    git_result(
        scheduler.update_config(config.into_inner()).await
            .map(|_| GitStatus::ok("Auto-pull configuration updated successfully", None))
            .map_err(|e| AppError::Internal(format!("Failed to update auto-pull configuration: {}", e)))
    )
}

/// GET /api/git/status
#[get("/git/status")]
pub async fn get_git_status(_admin: Admin) -> Json<GitRepoStatus> {
    let repo = match open_repo() {
        Ok(r) => r,
        Err(e) => return Json(GitRepoStatus::err(e.to_string())),
    };

    let current_branch = repo.head().ok()
        .and_then(|head| head.shorthand().map(|s| s.to_string()));

    let current_commit = repo.head().ok()
        .and_then(|head| head.target())
        .map(|oid| oid.to_string());

    let branch_name = current_branch.as_deref().unwrap_or("main");
    let remote_branch_name = format!("origin/{}", branch_name);

    let remote_commit = repo.find_branch(&remote_branch_name, git2::BranchType::Remote).ok()
        .and_then(|branch| branch.get().peel_to_commit().ok())
        .map(|commit| commit.id().to_string());

    let (behind_count, ahead_count) = if let (Some(local), Some(remote)) = (
        repo.head().ok().and_then(|h| h.target()),
        repo.find_branch(&remote_branch_name, git2::BranchType::Remote).ok()
            .and_then(|b| b.get().target())
    ) {
        let (ahead, behind) = repo.graph_ahead_behind(local, remote).unwrap_or((0, 0));
        (behind, ahead)
    } else {
        (0, 0)
    };

    let statuses = match repo.statuses(None) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get repository status: {}", e);
            return Json(GitRepoStatus {
                success: false,
                message: format!("Failed to get repository status: {}", e),
                current_branch, current_commit, remote_commit,
                behind_count: 0, ahead_count: 0,
                has_changes: false, has_staged_changes: false, has_unstaged_changes: false,
                changed_files: vec![], untracked_files: vec![],
            });
        }
    };

    let mut changed_files = Vec::new();
    let mut untracked_files = Vec::new();
    let mut has_staged_changes = false;
    let mut has_unstaged_changes = false;

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let flags = entry.status();

        let staged_status = if flags.contains(git2::Status::INDEX_NEW) { Some("staged_new") }
            else if flags.contains(git2::Status::INDEX_MODIFIED) { Some("staged_modified") }
            else if flags.contains(git2::Status::INDEX_DELETED) { Some("staged_deleted") }
            else if flags.contains(git2::Status::INDEX_RENAMED) { Some("staged_renamed") }
            else { None };

        if let Some(status) = staged_status {
            changed_files.push(GitFileStatus { path: path.clone(), status: status.to_string() });
            has_staged_changes = true;
        }

        let wt_status = if flags.contains(git2::Status::WT_NEW) { Some("untracked") }
            else if flags.contains(git2::Status::WT_MODIFIED) { Some("modified") }
            else if flags.contains(git2::Status::WT_DELETED) { Some("deleted") }
            else if flags.contains(git2::Status::WT_RENAMED) { Some("renamed") }
            else { None };

        if let Some(status) = wt_status {
            if status == "untracked" {
                untracked_files.push(path);
            } else {
                changed_files.push(GitFileStatus { path, status: status.to_string() });
            }
            has_unstaged_changes = true;
        }
    }

    let has_changes = has_staged_changes || has_unstaged_changes || !untracked_files.is_empty();

    Json(GitRepoStatus {
        success: true,
        message: "Repository status retrieved successfully".to_string(),
        current_branch, current_commit, remote_commit,
        behind_count, ahead_count,
        has_changes, has_staged_changes, has_unstaged_changes,
        changed_files, untracked_files,
    })
}

/// POST /api/git/commit
#[post("/git/commit", data = "<request>")]
pub async fn commit_changes(request: Json<CommitRequest>, _admin: Admin) -> Json<GitStatus> {
    info!("Committing changes with message: {}", request.message);
    git_result(commit_changes_inner(&request.message))
}

fn commit_changes_inner(message: &str) -> Result<GitStatus, AppError> {
    let repo = open_repo()?;

    let mut index = repo.index()
        .map_err(|e| AppError::Internal(format!("Failed to access repository index: {}", e)))?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| AppError::Internal(format!("Failed to stage changes: {}", e)))?;
    index.update_all(["*"].iter(), None)
        .map_err(|e| AppError::Internal(format!("Failed to update index: {}", e)))?;
    index.write()
        .map_err(|e| AppError::Internal(format!("Failed to write index: {}", e)))?;

    let tree_id = index.write_tree()
        .map_err(|e| AppError::Internal(format!("Failed to create commit tree: {}", e)))?;
    let tree = repo.find_tree(tree_id)
        .map_err(|e| AppError::Internal(format!("Failed to find commit tree: {}", e)))?;

    let parent_commit = repo.head().ok()
        .and_then(|head| head.target())
        .and_then(|oid| repo.find_commit(oid).ok());
    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    let signature = git2::Signature::now("Simple Web", "noreply@simple-web.local")
        .unwrap_or_else(|_| git2::Signature::now("Unknown", "unknown@local").unwrap());

    let oid = repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &parents)
        .map_err(|e| AppError::Internal(format!("Failed to create commit: {}", e)))?;

    info!("Commit created successfully: {}", oid);
    Ok(GitStatus::ok("Changes committed successfully", Some(oid.to_string())))
}

/// POST /api/git/push
#[post("/git/push")]
pub async fn push_repo(_admin: Admin) -> Json<GitStatus> {
    info!("Pushing local commits to remote repository");
    git_result(push_repo_inner().await)
}

async fn push_repo_inner() -> Result<GitStatus, AppError> {
    // Load credentials before any git2 operations (git2 types aren't Send)
    let creds = load_git_credentials().await;

    let repo = open_repo()?;
    ensure_clean_workdir(&repo, "push")?;

    let branch_name = current_branch_name(&repo)?;
    let mut remote = repo.find_remote("origin")
        .map_err(|e| AppError::Internal(format!("No remote 'origin' found: {}", e)))?;

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    let mut push_opts = creds.as_ref().map(make_push_options);
    remote.push(&[&refspec], push_opts.as_mut())
        .map_err(|e| AppError::Internal(format!(
            "Failed to push to remote: {}. Check if you have push permissions.", e
        )))?;

    Ok(GitStatus::ok("Successfully pushed commits to remote repository", head_commit_hash(&repo)))
}

/// POST /api/git/force-pull
#[post("/git/force-pull")]
pub async fn force_pull_repo(_admin: Admin) -> Json<GitStatus> {
    info!("Force pulling - this will overwrite local changes");
    git_result(force_pull_inner().await)
}

async fn force_pull_inner() -> Result<GitStatus, AppError> {
    // Load credentials before any git2 operations (git2 types aren't Send)
    let creds = load_git_credentials().await;

    let repo = open_repo()?;
    let mut result = fetch_and_reset(&repo, &creds)?;
    result.message = "Successfully force pulled - local changes discarded".to_string();
    Ok(result)
}
