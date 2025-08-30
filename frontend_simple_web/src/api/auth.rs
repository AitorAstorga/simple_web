// frontend_simple_web/src/api/auth.rs
use gloo::storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use gloo::net::http::Request;

const TOKEN_KEY: &str = "auth_token";

#[derive(Serialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

/// Store the raw token in localStorage
pub fn set_token(token: &str) {
    LocalStorage::set(TOKEN_KEY, token)
        .expect("failed to write auth token to localStorage");
}

/// Retrieve the raw token (if any) and check if it's still valid
pub fn get_token() -> String {
    LocalStorage::get::<String>(TOKEN_KEY).ok().unwrap_or("".to_owned())
}

/// Clear all authentication data from localStorage
pub fn clear_auth_data() {
    LocalStorage::delete(TOKEN_KEY);
    // Clear any other user-related data if needed
    // Note: We keep api_url and editor_url as they're config, not auth data
}

/// Check if user is currently authenticated
pub fn is_authenticated() -> bool {
    !get_token().is_empty()
}

/// Handle API response and check for authentication errors
pub fn handle_auth_error(status: u16) -> bool {
    if status == 401 || status == 403 {
        // Token expired or invalid - clear auth data and redirect to login
        clear_auth_data();
        
        // Force page reload to trigger auth guard redirect
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/");
        }
        true // Indicates auth error was handled
    } else {
        false // Not an auth error
    }
}

/// Logout user by clearing all auth data
pub fn logout() {
    clear_auth_data();
    
    // Redirect to login page
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/");
    }
}

/// Login with password and get a token
pub async fn login(password: &str) -> Result<String, String> {
    let api_url = crate::config_file::get_env_var("API_URL");
    let login_request = LoginRequest {
        password: password.to_string(),
    };

    let response = Request::post(&format!("{}/api/auth/", api_url))
        .header("Content-Type", "application/json")
        .json(&login_request)
        .map_err(|e| format!("Failed to create request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.ok() {
        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        set_token(&token_response.token);
        
        // Refresh config after successful login
        crate::config_file::load_config().await;
        
        Ok(token_response.token)
    } else {
        clear_auth_data(); // Clear any stale data on failed login
        Err(format!("Login failed: {}", response.status()))
    }
}