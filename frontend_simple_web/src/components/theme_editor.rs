// frontend_simple_web/src/components/theme_editor.rs
use yew::prelude::*;
use web_sys::HtmlInputElement;
use std::collections::HashMap;
use crate::highlighting::{
    highlighter::SyntaxHighlighter,
    config::HighlightConfig,
    themes::{ColorTheme, get_theme_by_name, get_available_themes},
};
use crate::api::themes::{api_list_themes, api_save_theme, api_delete_theme};
use gloo::console::log;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomTheme {
    pub name: String,
    pub colors: HashMap<String, String>,
}

impl Default for CustomTheme {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("keyword".to_string(), "#569cd6".to_string());
        colors.insert("string".to_string(), "#ce9178".to_string());
        colors.insert("comment".to_string(), "#6a9955".to_string());
        colors.insert("number".to_string(), "#b5cea8".to_string());
        colors.insert("tag".to_string(), "#4ec9b0".to_string());
        colors.insert("attr".to_string(), "#9cdcfe".to_string());
        colors.insert("url".to_string(), "#569cd6".to_string());
        colors.insert("color".to_string(), "#d7ba7d".to_string());
        colors.insert("pseudo".to_string(), "#dcdcaa".to_string());
        colors.insert("atrule".to_string(), "#c586c0".to_string());
        colors.insert("selector".to_string(), "#d7ba7d".to_string());
        colors.insert("constant".to_string(), "#4fc1ff".to_string());
        
        Self {
            name: "custom".to_string(),
            colors,
        }
    }
}

impl CustomTheme {
    pub fn to_color_theme(&self) -> ColorTheme {
        ColorTheme {
            name: self.name.clone(),
            colors: self.colors.clone(),
        }
    }
    
    pub fn from_color_theme(theme: &ColorTheme) -> Self {
        Self {
            name: theme.name.clone(),
            colors: theme.colors.clone(),
        }
    }
    
    pub fn save_to_storage(&self) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = storage.set_item(&format!("custom_theme_{}", self.name), &json);
                }
            }
        }
    }
    
    pub fn save_to_api<F>(&self, callback: Option<F>)
    where
        F: Fn(Result<String, String>) + 'static,
    {
        let api_theme = crate::api::themes::CustomTheme {
            name: self.name.clone(),
            colors: self.colors.clone(),
        };
        api_save_theme(api_theme, callback);
    }
    
    pub fn load_from_storage(name: &str) -> Option<Self> {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item(&format!("custom_theme_{}", name)) {
                    return serde_json::from_str(&json).ok();
                }
            }
        }
        None
    }
    
    pub fn list_custom_themes() -> Vec<String> {
        let mut themes = Vec::new();
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(length) = storage.length() {
                    for i in 0..length {
                        if let Ok(Some(key)) = storage.key(i) {
                            if key.starts_with("custom_theme_") {
                                let theme_name = key.strip_prefix("custom_theme_").unwrap();
                                themes.push(theme_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        themes
    }
    
    pub fn delete_from_storage(name: &str) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item(&format!("custom_theme_{}", name));
            }
        }
    }
    
    pub fn delete_from_api<F>(name: &str, callback: Option<F>)
    where
        F: Fn(Result<String, String>) + 'static,
    {
        api_delete_theme(name.to_string(), callback);
    }
    
    
    pub fn list_from_api<F>(callback: F)
    where
        F: Fn(Vec<String>) + 'static,
    {
        api_list_themes(Some(move |result| {
            match result {
                Ok(themes) => callback(themes),
                Err(_) => callback(Vec::new()),
            }
        }));
    }
}

#[derive(Properties, PartialEq)]
pub struct ThemeEditorProps {
    #[prop_or_default]
    pub onchange: Callback<String>,
}

#[function_component(ThemeEditor)]
pub fn theme_editor(props: &ThemeEditorProps) -> Html {
    let config = use_state(|| HighlightConfig::load());
    let current_theme = use_state(|| {
        if let Some(custom) = CustomTheme::load_from_storage(&config.theme) {
            custom
        } else {
            let builtin_theme = get_theme_by_name(&config.theme);
            CustomTheme::from_color_theme(&builtin_theme)
        }
    });
    let theme_name_input = use_state(|| current_theme.name.clone());
    let editing_mode = use_state(|| false);
    let custom_themes = use_state(|| CustomTheme::list_custom_themes());
    let api_themes = use_state(|| Vec::<String>::new());
    let sync_status = use_state(|| None::<String>);
    
    let available_builtin_themes = get_available_themes();
    
    // Load API themes on component mount
    {
        let api_themes = api_themes.clone();
        let sync_status = sync_status.clone();
        
        use_effect_with((), move |_| {
            CustomTheme::list_from_api({
                let api_themes = api_themes.clone();
                let sync_status = sync_status.clone();
                move |themes| {
                    api_themes.set(themes);
                    sync_status.set(Some("✅ Connected to server".to_string()));
                }
            });
            || {}
        });
    }
    
    let sample_code = r#"function greet(name: string) {
    // A simple greeting function
    const message = `Hello, ${name}!`;
    return message;
}

/* Multi-line comment example */
class UserService {
    private users: User[] = [];
    
    async fetchUsers(): Promise<User[]> {
        const response = await fetch('/api/users');
        return response.json();
    }
}

#[derive(Debug)]
struct Config {
    theme: String,
    enabled: bool,
}

.highlight-demo {
    background: #1e1e1e;
    color: #d4d4d4;
    border-radius: 4px;
    padding: 1rem;
}

@media (max-width: 768px) {
    .demo { font-size: 14px; }
}"#;
    
    let create_color_input = |color_type: &'static str, label: &'static str| {
        let current_theme = current_theme.clone();
        let color_value = current_theme.colors.get(color_type).cloned().unwrap_or_default();
        
        let on_color_change = {
            let current_theme = current_theme.clone();
            let color_type = color_type.to_string();
            
            Callback::from(move |e: Event| {
                if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                    let mut new_theme = (*current_theme).clone();
                    new_theme.colors.insert(color_type.clone(), input.value());
                    current_theme.set(new_theme);
                }
            })
        };
        
        html! {
            <div class="flex items-center gap-2 mb-2">
                <label class="w-20 text-sm font-medium">{label}</label>
                <input 
                    type="color"
                    value={color_value.clone()}
                    onchange={on_color_change.clone()}
                    class="w-12 h-8 border rounded cursor-pointer"
                />
                <input 
                    type="text"
                    value={color_value}
                    onchange={on_color_change}
                    class="input flex-1 text-sm font-mono"
                    placeholder="#ffffff"
                />
            </div>
        }
    };
    
    let on_theme_select = {
        let current_theme = current_theme.clone();
        let theme_name_input = theme_name_input.clone();
        let config = config.clone();
        let onchange = props.onchange.clone();
        
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let theme_name = select.value();
            
            if theme_name.starts_with("custom:") {
                let custom_name = theme_name.strip_prefix("custom:").unwrap();
                if let Some(custom) = CustomTheme::load_from_storage(custom_name) {
                    current_theme.set(custom.clone());
                    theme_name_input.set(custom.name.clone());
                    
                    // Update config and save
                    let mut new_config = (*config).clone();
                    new_config.theme = custom_name.to_string();
                    new_config.save();
                    config.set(new_config);
                    onchange.emit(custom_name.to_string());
                }
            } else {
                let builtin_theme = get_theme_by_name(&theme_name);
                let custom = CustomTheme::from_color_theme(&builtin_theme);
                current_theme.set(custom.clone());
                theme_name_input.set(custom.name.clone());
                
                // Update config and save
                let mut new_config = (*config).clone();
                new_config.theme = theme_name.clone();
                new_config.save();
                config.set(new_config);
                onchange.emit(theme_name.clone());
            }
        })
    };
    
    let on_theme_name_change = {
        let theme_name_input = theme_name_input.clone();
        
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                theme_name_input.set(input.value());
            }
        })
    };
    
    let save_custom_theme = {
        let current_theme = current_theme.clone();
        let theme_name_input = theme_name_input.clone();
        let custom_themes = custom_themes.clone();
        let api_themes = api_themes.clone();
        let config = config.clone();
        let onchange = props.onchange.clone();
        let sync_status = sync_status.clone();
        
        Callback::from(move |_| {
            let name = (*theme_name_input).clone();
            if !name.trim().is_empty() {
                let mut theme = (*current_theme).clone();
                theme.name = name.clone();
                
                // Save to localStorage first (immediate)
                theme.save_to_storage();
                
                // Update local custom themes list
                let mut themes = (*custom_themes).clone();
                if !themes.contains(&name) {
                    themes.push(name.clone());
                }
                custom_themes.set(themes);
                
                // Update config
                let mut new_config = (*config).clone();
                new_config.theme = name.clone();
                new_config.save();
                config.set(new_config);
                onchange.emit(name.clone());
                
                // Save to API (with fallback)
                let api_themes_clone = api_themes.clone();
                let sync_status_clone = sync_status.clone();
                let name_clone = name.clone();
                
                theme.save_to_api(Some(move |result| {
                    match result {
                        Ok(_) => {
                            // Update API themes list
                            let mut api_list = (*api_themes_clone).clone();
                            if !api_list.contains(&name_clone) {
                                api_list.push(name_clone.clone());
                            }
                            api_themes_clone.set(api_list);
                            sync_status_clone.set(Some("✅ Saved to server".to_string()));
                            log!("Theme saved to server successfully");
                        },
                        Err(e) => {
                            sync_status_clone.set(Some(format!("⚠️ Server save failed: {}", e)));
                            log!("Failed to save theme to server:", &e);
                        }
                    }
                }));
                
                log!("Saved custom theme:", &name);
            }
        })
    };
    
    let delete_custom_theme = {
        let custom_themes = custom_themes.clone();
        let api_themes = api_themes.clone();
        let theme_name_input = theme_name_input.clone();
        let sync_status = sync_status.clone();
        
        Callback::from(move |_| {
            let name = (*theme_name_input).clone();
            if custom_themes.contains(&name) || api_themes.contains(&name) {
                // Delete from localStorage
                CustomTheme::delete_from_storage(&name);
                
                // Update local themes list
                let mut themes = (*custom_themes).clone();
                themes.retain(|t| t != &name);
                custom_themes.set(themes);
                
                // Delete from API
                let api_themes_clone = api_themes.clone();
                let sync_status_clone = sync_status.clone();
                let name_clone = name.clone();
                
                CustomTheme::delete_from_api(&name, Some(move |result| {
                    match result {
                        Ok(_) => {
                            // Update API themes list
                            let mut api_list = (*api_themes_clone).clone();
                            api_list.retain(|t| t != &name_clone);
                            api_themes_clone.set(api_list);
                            sync_status_clone.set(Some("✅ Deleted from server".to_string()));
                            log!("Theme deleted from server successfully");
                        },
                        Err(e) => {
                            sync_status_clone.set(Some(format!("⚠️ Server delete failed: {}", e)));
                            log!("Failed to delete theme from server:", &e);
                        }
                    }
                }));
                
                log!("Deleted custom theme:", &name);
            }
        })
    };
    
    let toggle_editing = {
        let editing_mode = editing_mode.clone();
        
        Callback::from(move |_| {
            editing_mode.set(!*editing_mode);
        })
    };
    
    let current_theme_clone = (*current_theme).clone();
    let theme_for_highlighting = current_theme_clone.to_color_theme();
    let highlighter = SyntaxHighlighter::with_theme(&theme_for_highlighting.name);
    let highlighted_sample = highlighter.highlight(sample_code);
    
    // Combine all custom themes for the dropdown
    let all_custom_themes = {
        let mut themes = custom_themes.iter().cloned().collect::<Vec<String>>();
        let mut api_only: Vec<String> = api_themes.iter()
            .filter(|t| !custom_themes.contains(t))
            .cloned()
            .collect();
        themes.append(&mut api_only);
        themes.sort();
        themes
    };
    
    html! {
        <div class="theme-editor space-y-6">
            // Theme Selection
            <div>
                <label for="theme-select" class="block font-medium mb-2">
                    {"Select Base Theme"}
                </label>
                <select 
                    id="theme-select"
                    class="input w-full"
                    onchange={on_theme_select}
                >
                    <optgroup label="Built-in Themes">
                        {for available_builtin_themes.iter().map(|theme| {
                            html! {
                                <option value={theme.name.clone()}>
                                    {theme.name.replace('_', " ").to_uppercase()}
                                </option>
                            }
                        })}
                    </optgroup>
                    { if !all_custom_themes.is_empty() {
                        html! {
                            <optgroup label="Custom Themes">
                                {for all_custom_themes.iter().map(|theme_name| {
                                        let is_local = custom_themes.contains(theme_name);
                                        let is_server = api_themes.contains(theme_name);
                                        let status_icon = if is_local && is_server {
                                            "✅ " // synced
                                        } else if is_local {
                                            "💾 " // local only
                                        } else {
                                            "☁️ " // server only
                                        };
                                        
                                        html! {
                                            <option value={format!("custom:{}", theme_name)}>
                                                {format!("{}{}", status_icon, theme_name)}
                                            </option>
                                        }
                                    })}
                                </optgroup>
                            }
                        } else {
                            html! {}
                        }
                    }
                </select>
            </div>
            
            // Theme Name and Actions
            <div class="flex items-center gap-2">
                <div class="flex-1">
                    <label class="block font-medium mb-2">{"Theme Name"}</label>
                    <input 
                        type="text"
                        value={(*theme_name_input).clone()}
                        onchange={on_theme_name_change}
                        class="input w-full"
                        placeholder="My Custom Theme"
                    />
                </div>
                <div class="flex gap-2 pt-6">
                    <button 
                        class="btn btn-secondary"
                        onclick={toggle_editing}
                    >
                        {if *editing_mode { "Hide Editor" } else { "Edit Colors" }}
                    </button>
                    <button 
                        class="btn btn-primary"
                        onclick={save_custom_theme}
                    >
                        {"Save Theme"}
                    </button>
                    { if custom_themes.contains(&*theme_name_input) || api_themes.contains(&*theme_name_input) {
                        html! {
                            <button 
                                class="btn btn-danger"
                                onclick={delete_custom_theme}
                            >
                                {"Delete"}
                            </button>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </div>
            
            // Color Editor
            { if *editing_mode {
                html! {
                    <div class="bg-surface p-4 rounded border">
                        <h3 class="font-medium mb-4">{"Customize Colors"}</h3>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <h4 class="font-medium mb-2 text-sm">{"Syntax Elements"}</h4>
                                {create_color_input("keyword", "Keywords")}
                                {create_color_input("string", "Strings")}
                                {create_color_input("comment", "Comments")}
                                {create_color_input("number", "Numbers")}
                                {create_color_input("constant", "Constants")}
                                {create_color_input("attr", "Attributes")}
                            </div>
                            <div>
                                <h4 class="font-medium mb-2 text-sm">{"Web Elements"}</h4>
                                {create_color_input("tag", "HTML Tags")}
                                {create_color_input("selector", "CSS Selectors")}
                                {create_color_input("pseudo", "Pseudo Classes")}
                                {create_color_input("atrule", "CSS At-Rules")}
                                {create_color_input("color", "Color Values")}
                                {create_color_input("url", "URLs")}
                            </div>
                        </div>
                    </div>
                }
            } else {
                html! {}
            }}
            
            // Preview
            <div>
                <label class="block font-medium mb-2">{"Preview"}</label>
                <div class="border rounded p-4 bg-card font-mono text-sm overflow-y-auto" style="max-height: 300px;">
                    {Html::from_html_unchecked(highlighted_sample.into())}
                </div>
            </div>
            
            // Sync Status
            { if let Some(ref status) = *sync_status {
                html! {
                    <div class="p-3 rounded border" style={
                        if status.starts_with("✅") {
                            "background-color: #dcfce7; color: #166534; border-color: #bbf7d0;"
                        } else {
                            "background-color: #fefce8; color: #ca8a04; border-color: #fde68a;"
                        }
                    }>
                        <div class="text-sm font-medium">{status}</div>
                    </div>
                }
            } else {
                html! {}
            }}
            
        </div>
    }
}