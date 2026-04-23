// frontend_simple_web/src/api/themes.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::client::{self, Method};

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

pub fn api_list_themes<F: Fn(Result<Vec<String>, String>) + 'static>(callback: Option<F>) {
    client::spawn_request::<ThemeListResponse, _>(
        Method::Get,
        "/api/themes".into(),
        None,
        callback.map(|cb| {
            move |result: Result<ThemeListResponse, String>| {
                cb(result.map(|r| r.themes));
            }
        }),
    );
}

pub fn api_save_theme<F: Fn(Result<String, String>) + 'static>(theme: CustomTheme, callback: Option<F>) {
    let body = match serde_json::to_string(&theme) {
        Ok(b) => b,
        Err(e) => {
            if let Some(cb) = callback { cb(Err(format!("Failed to serialize theme: {}", e))); }
            return;
        }
    };

    client::spawn_request::<ThemeResponse, _>(
        Method::Post,
        "/api/themes".into(),
        Some(body),
        callback.map(|cb| {
            move |result: Result<ThemeResponse, String>| {
                match result {
                    Ok(r) if r.success => cb(Ok(r.message)),
                    Ok(r) => cb(Err(r.message)),
                    Err(e) => cb(Err(e)),
                }
            }
        }),
    );
}

pub fn api_delete_theme<F: Fn(Result<String, String>) + 'static>(theme_name: String, callback: Option<F>) {
    let url = format!("/api/themes/{}", theme_name);

    client::spawn_request::<ThemeResponse, _>(
        Method::Delete,
        url,
        None,
        callback.map(|cb| {
            move |result: Result<ThemeResponse, String>| {
                match result {
                    Ok(r) if r.success => cb(Ok(r.message)),
                    Ok(r) => cb(Err(r.message)),
                    Err(e) => cb(Err(e)),
                }
            }
        }),
    );
}
