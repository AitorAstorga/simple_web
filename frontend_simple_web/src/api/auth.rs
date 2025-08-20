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

/// Retrieve the raw token (if any)
pub fn get_token() -> String {
    LocalStorage::get::<String>(TOKEN_KEY).ok().unwrap_or("".to_owned())
}

/// Clear the stored token
pub fn clear_token() {
    LocalStorage::delete(TOKEN_KEY);
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
        Ok(token_response.token)
    } else {
        Err(format!("Login failed: {}", response.status()))
    }
}