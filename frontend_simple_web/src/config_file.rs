use gloo::{console::error, net::http::Request};
use once_cell::sync::OnceCell;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FrontendConfig {
    pub api_url: String,
    pub editor_url: String,
}

static CONFIG: OnceCell<FrontendConfig> = OnceCell::new();

pub async fn load_config() {
    if CONFIG.get().is_none() {
        let response = Request::get("/config/config.json")
            .send()
            .await
            .expect("Failed to fetch config");

        let config: FrontendConfig = response
            .json()
            .await
            .expect("Failed to parse config.json");

        CONFIG.set(config).ok();
    }
}

pub fn get_env_var(key: &str) -> String {
    let value = CONFIG.get().and_then(|cfg| match key {
        "API_URL" => Some(cfg.api_url.clone()),
        "EDITOR_URL" => Some(cfg.editor_url.clone()),
        _ => None,
    });

    if value.is_none() {
        error!("Failed to get env var: {key}");
    }

    value.unwrap_or_default()
}
