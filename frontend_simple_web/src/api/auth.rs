// frontend_simple_web/src/api/auth.rs
use gloo::storage::{LocalStorage, Storage};
const TOKEN_KEY: &str = "auth_token";

/// Store the raw token in localStorage
pub fn set_token(token: &str) {
    LocalStorage::set(TOKEN_KEY, token)
        .expect("failed to write auth token to localStorage");
}

/// Retrieve the raw token (if any)
pub fn get_token() -> String {
    LocalStorage::get::<String>(TOKEN_KEY).ok().unwrap_or("".to_owned())
}