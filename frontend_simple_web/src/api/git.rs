// frontend_simple_web/src/api/git.rs
use gloo::{console::{error, log}, net::http::Request};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::{api::auth::get_token, config_file::get_env_var};

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

pub fn api_git_pull(callback: Option<impl Fn(Result<GitStatus, String>) + 'static>) {
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/git/pull");
        
        let req = Request::post(&url)
            .header("Authorization", &auth);
        
        match req.send().await {
            Ok(response) => {
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