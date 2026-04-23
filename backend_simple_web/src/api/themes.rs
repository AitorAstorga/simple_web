// backend_simple_web/src/api/themes.rs
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::tokio::fs;
use std::collections::HashMap;
use std::path::Path;
use prisma_auth::backend::AuthGuard as Admin;

use super::error::AppError;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CustomTheme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ThemeListResponse {
    pub themes: Vec<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ThemeResponse {
    pub success: bool,
    pub message: String,
    pub theme: Option<CustomTheme>,
}

// Store themes outside the Git repository to avoid conflicts
const THEMES_DIR: &str = "/app/data/themes";

async fn ensure_themes_dir() -> Result<(), AppError> {
    fs::create_dir_all(THEMES_DIR).await?;

    // Migrate themes from old location if it exists
    let old_themes_dir = "/public_site/.themes";
    if fs::metadata(old_themes_dir).await.is_ok() {
        let mut entries = fs::read_dir(old_themes_dir).await
            .map_err(|e| AppError::Internal(format!("Failed to read old themes dir: {}", e)))?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    let old_path = entry.path();
                    let new_path = Path::new(THEMES_DIR).join(file_name);
                    let _ = fs::copy(&old_path, &new_path).await;
                }
            }
        }
        // Remove old directory after migration
        let _ = fs::remove_dir_all(old_themes_dir).await;
    }

    Ok(())
}

fn theme_file_path(theme_name: &str) -> String {
    format!("{}/{}.json", THEMES_DIR, theme_name)
}

/// Get list of all custom themes
#[get("/themes")]
pub async fn list_themes(_admin: Admin) -> Result<Json<ThemeListResponse>, AppError> {
    ensure_themes_dir().await?;

    let mut themes = Vec::new();

    let mut entries = fs::read_dir(THEMES_DIR).await?;
    while let Ok(Some(entry)) = entries.next_entry().await {
        if let Some(file_name) = entry.file_name().to_str() {
            if let Some(theme_name) = file_name.strip_suffix(".json") {
                themes.push(theme_name.to_string());
            }
        }
    }

    themes.sort();
    Ok(Json(ThemeListResponse { themes }))
}

/// Get a specific custom theme
#[get("/themes/<theme_name>")]
pub async fn get_theme(_admin: Admin, theme_name: &str) -> Result<Json<ThemeResponse>, AppError> {
    ensure_themes_dir().await?;

    let file_path = theme_file_path(theme_name);

    let content = fs::read_to_string(&file_path).await.map_err(|_| {
        AppError::NotFound(format!("Theme '{}' not found", theme_name))
    })?;

    let theme: CustomTheme = serde_json::from_str(&content).map_err(|e| {
        AppError::Internal(format!("Invalid theme file format for '{}': {}", theme_name, e))
    })?;

    Ok(Json(ThemeResponse {
        success: true,
        message: "Theme retrieved successfully".to_string(),
        theme: Some(theme),
    }))
}

/// Save a custom theme
#[post("/themes", format = "json", data = "<theme>")]
pub async fn save_theme(_admin: Admin, theme: Json<CustomTheme>) -> Result<Json<ThemeResponse>, AppError> {
    ensure_themes_dir().await?;

    if theme.name.is_empty() {
        return Err(AppError::BadRequest("Theme name cannot be empty".into()));
    }

    if !theme.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::BadRequest(
            "Theme name can only contain letters, numbers, hyphens, and underscores".into(),
        ));
    }

    let file_path = theme_file_path(&theme.name);
    let json_content = serde_json::to_string_pretty(&*theme)?;
    fs::write(&file_path, json_content).await?;

    Ok(Json(ThemeResponse {
        success: true,
        message: format!("Theme '{}' saved successfully", theme.name),
        theme: Some(theme.into_inner()),
    }))
}

/// Delete a custom theme
#[delete("/themes/<theme_name>")]
pub async fn delete_theme(_admin: Admin, theme_name: &str) -> Result<Json<ThemeResponse>, AppError> {
    ensure_themes_dir().await?;

    let file_path = theme_file_path(theme_name);

    if fs::metadata(&file_path).await.is_err() {
        return Err(AppError::NotFound(format!("Theme '{}' not found", theme_name)));
    }

    fs::remove_file(&file_path).await.map_err(|e| {
        AppError::Internal(format!("Failed to delete theme '{}': {}", theme_name, e))
    })?;

    Ok(Json(ThemeResponse {
        success: true,
        message: format!("Theme '{}' deleted successfully", theme_name),
        theme: None,
    }))
}
