// backend_simple_web/src/api/themes.rs
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{get, post, delete, http::Status};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use prisma_auth::backend::AuthGuard as Admin;

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

fn ensure_themes_dir() -> Result<(), std::io::Error> {
    // Create new themes directory outside Git repository
    fs::create_dir_all(THEMES_DIR)?;
    
    // Migrate themes from old location if it exists
    let old_themes_dir = "/public_site/.themes";
    if Path::new(old_themes_dir).exists() {
        // Try to migrate existing themes
        if let Ok(entries) = fs::read_dir(old_themes_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        let old_path = entry.path();
                        let new_path = Path::new(THEMES_DIR).join(file_name);
                        let _ = fs::copy(&old_path, &new_path);
                    }
                }
            }
        }
        // Remove old directory after migration
        let _ = fs::remove_dir_all(old_themes_dir);
    }
    
    Ok(())
}

fn theme_file_path(theme_name: &str) -> String {
    format!("{}/{}.json", THEMES_DIR, theme_name)
}

/// Get list of all custom themes
/// ### Returns
/// - `Json<ThemeListResponse>`: List of available theme names
/// ### Examples
/// - `curl -H "Authorization: <token>" http://localhost:8000/api/themes`
#[get("/themes")]
pub fn list_themes(_admin: Admin) -> Result<Json<ThemeListResponse>, Status> {
    ensure_themes_dir().map_err(|_| Status::InternalServerError)?;
    
    let mut themes = Vec::new();
    
    if let Ok(entries) = fs::read_dir(THEMES_DIR) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    let theme_name = file_name.strip_suffix(".json").unwrap();
                    themes.push(theme_name.to_string());
                }
            }
        }
    }
    
    themes.sort();
    Ok(Json(ThemeListResponse { themes }))
}

/// Get a specific custom theme
/// ### Arguments
/// - `theme_name`: Name of the theme to retrieve
/// ### Returns
/// - `Json<ThemeResponse>`: Theme data or error message
/// ### Examples
/// - `curl -H "Authorization: <token>" http://localhost:8000/api/themes/my-theme`
#[get("/themes/<theme_name>")]
pub fn get_theme(_admin: Admin, theme_name: &str) -> Result<Json<ThemeResponse>, Status> {
    ensure_themes_dir().map_err(|_| Status::InternalServerError)?;
    
    let file_path = theme_file_path(theme_name);
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            match serde_json::from_str::<CustomTheme>(&content) {
                Ok(theme) => Ok(Json(ThemeResponse {
                    success: true,
                    message: "Theme retrieved successfully".to_string(),
                    theme: Some(theme),
                })),
                Err(_) => Ok(Json(ThemeResponse {
                    success: false,
                    message: "Invalid theme file format".to_string(),
                    theme: None,
                })),
            }
        },
        Err(_) => Ok(Json(ThemeResponse {
            success: false,
            message: "Theme not found".to_string(),
            theme: None,
        })),
    }
}

/// Save a custom theme
/// ### Arguments
/// - `theme`: JSON with theme data
/// ### Returns
/// - `Json<ThemeResponse>`: Success/failure message
/// ### Examples
/// - `curl -i -X POST -H "Authorization: <token>" -H "Content-Type: application/json" -d '{"name":"my-theme","colors":{"keyword":"#ff0000"}}' http://localhost:8000/api/themes`
#[post("/themes", format = "json", data = "<theme>")]
pub fn save_theme(_admin: Admin, theme: Json<CustomTheme>) -> Result<Json<ThemeResponse>, Status> {
    ensure_themes_dir().map_err(|_| Status::InternalServerError)?;
    
    // Validate theme name (alphanumeric, hyphens, underscores only)
    if !theme.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Ok(Json(ThemeResponse {
            success: false,
            message: "Theme name can only contain letters, numbers, hyphens, and underscores".to_string(),
            theme: None,
        }));
    }
    
    if theme.name.is_empty() {
        return Ok(Json(ThemeResponse {
            success: false,
            message: "Theme name cannot be empty".to_string(),
            theme: None,
        }));
    }
    
    let file_path = theme_file_path(&theme.name);
    
    match serde_json::to_string_pretty(&*theme) {
        Ok(json_content) => {
            match fs::write(&file_path, json_content) {
                Ok(_) => Ok(Json(ThemeResponse {
                    success: true,
                    message: format!("Theme '{}' saved successfully", theme.name),
                    theme: Some(theme.into_inner()),
                })),
                Err(_) => Ok(Json(ThemeResponse {
                    success: false,
                    message: "Failed to save theme file".to_string(),
                    theme: None,
                })),
            }
        },
        Err(_) => Ok(Json(ThemeResponse {
            success: false,
            message: "Failed to serialize theme data".to_string(),
            theme: None,
        })),
    }
}

/// Delete a custom theme
/// ### Arguments
/// - `theme_name`: Name of the theme to delete
/// ### Returns
/// - `Json<ThemeResponse>`: Success/failure message
/// ### Examples
/// - `curl -i -X DELETE -H "Authorization: <token>" http://localhost:8000/api/themes/my-theme`


#[delete("/themes/<theme_name>")]
pub fn delete_theme(_admin: Admin, theme_name: &str) -> Result<Json<ThemeResponse>, Status> {
    ensure_themes_dir().map_err(|_| Status::InternalServerError)?;
    
    let file_path = theme_file_path(theme_name);
    
    if !Path::new(&file_path).exists() {
        return Ok(Json(ThemeResponse {
            success: false,
            message: "Theme not found".to_string(),
            theme: None,
        }));
    }
    
    match fs::remove_file(&file_path) {
        Ok(_) => Ok(Json(ThemeResponse {
            success: true,
            message: format!("Theme '{}' deleted successfully", theme_name),
            theme: None,
        })),
        Err(_) => Ok(Json(ThemeResponse {
            success: false,
            message: "Failed to delete theme file".to_string(),
            theme: None,
        })),
    }
}