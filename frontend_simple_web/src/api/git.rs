// frontend_simple_web/src/api/git.rs
use gloo::{console::{error, log}, net::http::Request};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::{api::auth::{get_token, handle_auth_error}, config_file::get_env_var};

#[derive(Serialize)]
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
pub struct CommitRequest {
    message: String,
}

// Helper method to reload the page
fn reload() { let _ = web_sys::window().map(|w| w.location().reload()); }

pub fn api_git_setup(config: GitRepoConfig, callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/setup");
        let body = match serde_json::to_string(&config) {
            Ok(b) => b,
            Err(e) => {
                error!(&format!("Failed to serialize git config: {}", e));
                if let Some(cb) = callback {
                    cb(Err(format!("Failed to serialize config: {}", e)));
                }
                return;
            }
        };
        
        let req = Request::post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(body);
        
        let response = match req {
            Ok(r) => r.send().await,
            Err(e) => {
                error!(&format!("Failed to build request: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Failed to build request".to_string()));
                }
                return;
            }
        };
        
        match response {
            Ok(response) => {
                // Check for authentication errors first
                if handle_auth_error(response.status()) {
                    if let Some(cb) = callback {
                        cb(Err("Authentication failed".to_string()));
                    }
                    return;
                }
                
                match response.json::<GitStatus>().await {
                    Ok(status) => {
                        log!(&format!("Git setup result: {}", status.message));
                        if let Some(cb) = callback {
                            cb(Ok(status));
                        } else {
                            reload();
                        }
                    }
                    Err(e) => {
                        error!(&format!("Failed to parse response: {:?}", e));
                        if let Some(cb) = callback {
                            cb(Err("Failed to parse response".to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!(&format!("Request failed: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Request failed".to_string()));
                }
            }
        }
    });
}

pub fn api_git_test(config: GitRepoConfig, callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/test");
        let body = match serde_json::to_string(&config) {
            Ok(b) => b,
            Err(e) => {
                error!(&format!("Failed to serialize git config: {}", e));
                if let Some(cb) = callback {
                    cb(Err(format!("Failed to serialize config: {}", e)));
                }
                return;
            }
        };
        
        let req = Request::post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(body);
        
        let response = match req {
            Ok(r) => r.send().await,
            Err(e) => {
                error!(&format!("Failed to build test request: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Failed to build request".to_string()));
                }
                return;
            }
        };
        
        match response {
            Ok(response) => {
                // Check for authentication errors first
                if handle_auth_error(response.status()) {
                    if let Some(cb) = callback {
                        cb(Err("Authentication failed".to_string()));
                    }
                    return;
                }
                
                match response.json::<GitStatus>().await {
                    Ok(status) => {
                        log!(&format!("Git test result: {}", status.message));
                        if let Some(cb) = callback {
                            cb(Ok(status));
                        }
                    }
                    Err(e) => {
                        error!(&format!("Failed to parse test response: {:?}", e));
                        if let Some(cb) = callback {
                            cb(Err("Failed to parse response".to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!(&format!("Test request failed: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Request failed".to_string()));
                }
            }
        }
    });
}

pub fn api_git_pull(callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/pull");
        
        let req = Request::post(&url)
            .header("Authorization", &auth);
        
        match req.send().await {
            Ok(response) => {
                // Check for authentication errors first
                if handle_auth_error(response.status()) {
                    if let Some(cb) = callback {
                        cb(Err("Authentication failed".to_string()));
                    }
                    return;
                }
                
                match response.json::<GitStatus>().await {
                    Ok(status) => {
                        log!(&format!("Git pull result: {}", status.message));
                        if let Some(cb) = callback {
                            cb(Ok(status));
                        } else {
                            reload();
                        }
                    }
                    Err(e) => {
                        error!(&format!("Failed to parse pull response: {:?}", e));
                        if let Some(cb) = callback {
                            cb(Err("Failed to parse response".to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!(&format!("Pull request failed: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Request failed".to_string()));
                }
            }
        }
    });
}

pub fn api_get_git_status(callback: Option<impl Fn(Result<GitRepoStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/status");
        
        let req = Request::get(&url)
            .header("Authorization", &auth);
        
        match req.send().await {
            Ok(response) => {
                // Check for authentication errors first
                if handle_auth_error(response.status()) {
                    if let Some(cb) = callback {
                        cb(Err("Authentication failed".to_string()));
                    }
                    return;
                }
                
                match response.json::<GitRepoStatus>().await {
                    Ok(status) => {
                        log!(&format!("Git status result: has_changes={}, behind={}, ahead={}", 
                                      status.has_changes, status.behind_count, status.ahead_count));
                        if let Some(cb) = callback {
                            cb(Ok(status));
                        }
                    }
                    Err(e) => {
                        error!(&format!("Failed to parse git status response: {:?}", e));
                        if let Some(cb) = callback {
                            cb(Err("Failed to parse response".to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!(&format!("Git status request failed: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Request failed".to_string()));
                }
            }
        }
    });
}

pub fn api_commit_changes(message: String, callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/commit");
        let request = CommitRequest { message };
        let body = match serde_json::to_string(&request) {
            Ok(b) => b,
            Err(e) => {
                error!(&format!("Failed to serialize commit request: {}", e));
                if let Some(cb) = callback {
                    cb(Err(format!("Failed to serialize request: {}", e)));
                }
                return;
            }
        };
        
        let req = Request::post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(body);
        
        let response = match req {
            Ok(r) => r.send().await,
            Err(e) => {
                error!(&format!("Failed to build commit request: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Failed to build request".to_string()));
                }
                return;
            }
        };
        
        match response {
            Ok(response) => {
                // Check for authentication errors first
                if handle_auth_error(response.status()) {
                    if let Some(cb) = callback {
                        cb(Err("Authentication failed".to_string()));
                    }
                    return;
                }
                
                match response.json::<GitStatus>().await {
                    Ok(status) => {
                        log!(&format!("Commit result: {}", status.message));
                        if let Some(cb) = callback {
                            cb(Ok(status));
                        }
                    }
                    Err(e) => {
                        error!(&format!("Failed to parse commit response: {:?}", e));
                        if let Some(cb) = callback {
                            cb(Err("Failed to parse response".to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!(&format!("Commit request failed: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Request failed".to_string()));
                }
            }
        }
    });
}

pub fn api_push_changes(callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/push");
        
        let req = Request::post(&url)
            .header("Authorization", &auth);
        
        match req.send().await {
            Ok(response) => {
                // Check for authentication errors first
                if handle_auth_error(response.status()) {
                    if let Some(cb) = callback {
                        cb(Err("Authentication failed".to_string()));
                    }
                    return;
                }
                
                match response.json::<GitStatus>().await {
                    Ok(status) => {
                        log!(&format!("Push result: {}", status.message));
                        if let Some(cb) = callback {
                            cb(Ok(status));
                        }
                    }
                    Err(e) => {
                        error!(&format!("Failed to parse push response: {:?}", e));
                        if let Some(cb) = callback {
                            cb(Err("Failed to parse response".to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!(&format!("Push request failed: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Request failed".to_string()));
                }
            }
        }
    });
}

pub fn api_force_pull(callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/force-pull");
        
        let req = Request::post(&url)
            .header("Authorization", &auth);
        
        match req.send().await {
            Ok(response) => {
                // Check for authentication errors first
                if handle_auth_error(response.status()) {
                    if let Some(cb) = callback {
                        cb(Err("Authentication failed".to_string()));
                    }
                    return;
                }
                
                match response.json::<GitStatus>().await {
                    Ok(status) => {
                        log!(&format!("Force pull result: {}", status.message));
                        if let Some(cb) = callback {
                            cb(Ok(status));
                        } else {
                            reload();
                        }
                    }
                    Err(e) => {
                        error!(&format!("Failed to parse force pull response: {:?}", e));
                        if let Some(cb) = callback {
                            cb(Err("Failed to parse response".to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                error!(&format!("Force pull request failed: {:?}", e));
                if let Some(cb) = callback {
                    cb(Err("Request failed".to_string()));
                }
            }
        }
    });
}