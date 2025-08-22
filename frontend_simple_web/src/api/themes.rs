// frontend_simple_web/src/api/themes.rs
use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use crate::api::auth::{get_token, handle_auth_error};
use crate::config_file::get_env_var;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomTheme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

#[derive(Deserialize, Clone)]
pub struct ThemeListResponse {
    pub themes: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct ThemeResponse {
    pub success: bool,
    pub message: String,
    pub theme: Option<CustomTheme>,
}


/// Get list of all custom themes from the server
pub fn api_list_themes<F>(callback: Option<F>)
where
    F: Fn(Result<Vec<String>, String>) + 'static,
{
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/themes");
        
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
                
                match response.json::<ThemeListResponse>().await {
                    Ok(theme_response) => {
                        if let Some(cb) = callback {
                            cb(Ok(theme_response.themes));
                        }
                    },
                    Err(e) => {
                        if let Some(cb) = callback {
                            cb(Err(format!("Failed to parse response: {}", e)));
                        }
                    }
                }
            },
            Err(e) => {
                if let Some(cb) = callback {
                    cb(Err(format!("Request failed: {}", e)));
                }
            }
        }
    });
}


/// Save a custom theme to the server
pub fn api_save_theme<F>(theme: CustomTheme, callback: Option<F>)
where
    F: Fn(Result<String, String>) + 'static,
{
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/themes");
        
        let req = Request::post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .json(&theme);
        
        match req {
            Ok(request) => {
                match request.send().await {
                    Ok(response) => {
                        // Check for authentication errors first
                        if handle_auth_error(response.status()) {
                            if let Some(cb) = callback {
                                cb(Err("Authentication failed".to_string()));
                            }
                            return;
                        }
                        
                        match response.json::<ThemeResponse>().await {
                            Ok(theme_response) => {
                                if let Some(cb) = callback {
                                    if theme_response.success {
                                        cb(Ok(theme_response.message));
                                    } else {
                                        cb(Err(theme_response.message));
                                    }
                                }
                            },
                            Err(e) => {
                                if let Some(cb) = callback {
                                    cb(Err(format!("Failed to parse response: {}", e)));
                                }
                            }
                        }
                    },
                    Err(e) => {
                        if let Some(cb) = callback {
                            cb(Err(format!("Request failed: {}", e)));
                        }
                    }
                }
            },
            Err(e) => {
                if let Some(cb) = callback {
                    cb(Err(format!("Failed to serialize theme: {}", e)));
                }
            }
        }
    });
}

/// Delete a custom theme from the server
pub fn api_delete_theme<F>(theme_name: String, callback: Option<F>)
where
    F: Fn(Result<String, String>) + 'static,
{
    let api_url = get_env_var("API_URL");
    let auth = get_token();
    
    spawn_local(async move {
        let url = format!("{api_url}/api/themes/{theme_name}");
        
        let req = Request::delete(&url)
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
                
                match response.json::<ThemeResponse>().await {
                    Ok(theme_response) => {
                        if let Some(cb) = callback {
                            if theme_response.success {
                                cb(Ok(theme_response.message));
                            } else {
                                cb(Err(theme_response.message));
                            }
                        }
                    },
                    Err(e) => {
                        if let Some(cb) = callback {
                            cb(Err(format!("Failed to parse response: {}", e)));
                        }
                    }
                }
            },
            Err(e) => {
                if let Some(cb) = callback {
                    cb(Err(format!("Request failed: {}", e)));
                }
            }
        }
    });
}