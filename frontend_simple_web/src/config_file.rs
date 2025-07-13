use gloo::{console::error, net::http::Request};
use gloo::storage::{LocalStorage, Storage};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FrontendConfig {
    pub api_url: String,
    pub editor_url: String,
}

const API_URL: &str = "api_url";
const EDITOR_URL: &str = "editor_url";

pub async fn load_config() {
    let response = Request::get("/config/config.json")
        .send()
        .await
        .expect("Failed to fetch config");

    let config: FrontendConfig = response
        .json()
        .await
        .expect("Failed to parse config.json");

    LocalStorage::set(API_URL, config.api_url.clone())
        .expect("failed to write API_URL to localStorage");

    LocalStorage::set(EDITOR_URL, config.editor_url.clone())
        .expect("failed to write EDITOR_URL to localStorage");
}

pub fn get_env_var(key: &str) -> String {
    let value = match key {
        "API_URL" => LocalStorage::get(API_URL).ok().unwrap_or("".to_owned()),
        "EDITOR_URL" => LocalStorage::get(EDITOR_URL).ok().unwrap_or("".to_owned()),
        _ => "".to_owned(),
    };

    if value.is_empty() {
        error!("Failed to get env var: {key}");
    }

    value
}
