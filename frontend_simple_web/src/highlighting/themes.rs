// frontend_simple_web/src/highlighting/themes.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ColorTheme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

pub fn get_available_themes() -> Vec<ColorTheme> {
    vec![
        get_dark_theme(),
        get_light_theme(),
    ]
}

pub fn get_theme_by_name(name: &str) -> ColorTheme {
    match name {
        "light" => get_light_theme(),
        _ => get_dark_theme(), // default
    }
}

fn get_dark_theme() -> ColorTheme {
    let mut colors = HashMap::new();
    colors.insert("keyword".to_string(), "#ff6b6b".to_string());     // Bright red
    colors.insert("string".to_string(), "#ffd93d".to_string());      // Yellow
    colors.insert("number".to_string(), "#74c0fc".to_string());      // Light blue
    colors.insert("comment".to_string(), "#868e96".to_string());     // Gray
    colors.insert("tag".to_string(), "#51cf66".to_string());         // Green
    colors.insert("attr".to_string(), "#ff8cc8".to_string());        // Pink
    colors.insert("selector".to_string(), "#da77f2".to_string());    // Purple
    colors.insert("color".to_string(), "#20c997".to_string());       // Teal
    colors.insert("url".to_string(), "#fd7e14".to_string());         // Orange
    colors.insert("pseudo".to_string(), "#e599f7".to_string());      // Light purple
    colors.insert("atrule".to_string(), "#ff8787".to_string());      // Light red
    colors.insert("constant".to_string(), "#91a7ff".to_string());    // Light blue
    
    ColorTheme {
        name: "dark".to_string(),
        colors,
    }
}

fn get_light_theme() -> ColorTheme {
    let mut colors = HashMap::new();
    colors.insert("keyword".to_string(), "#d63384".to_string());     // Pink
    colors.insert("string".to_string(), "#198754".to_string());      // Green
    colors.insert("number".to_string(), "#0d6efd".to_string());      // Blue
    colors.insert("comment".to_string(), "#6c757d".to_string());     // Muted gray
    colors.insert("tag".to_string(), "#dc3545".to_string());         // Red
    colors.insert("attr".to_string(), "#6f42c1".to_string());        // Purple
    colors.insert("selector".to_string(), "#fd7e14".to_string());    // Orange
    colors.insert("color".to_string(), "#20c997".to_string());       // Teal
    colors.insert("url".to_string(), "#0dcaf0".to_string());         // Cyan
    colors.insert("pseudo".to_string(), "#6610f2".to_string());      // Indigo
    colors.insert("atrule".to_string(), "#e91e63".to_string());      // Pink
    colors.insert("constant".to_string(), "#495057".to_string());    // Dark gray
    
    ColorTheme {
        name: "light".to_string(),
        colors,
    }
}