// frontend_simple_web/src/components/theme_selector.rs
use yew::prelude::*;
use crate::highlighting::{
    themes::get_available_themes,
    highlighter::SyntaxHighlighter,
    config::HighlightConfig,
};
use gloo::console::log;

#[derive(Properties, PartialEq)]
pub struct ThemeSelectorProps {
    #[prop_or_default]
    pub onchange: Callback<String>,
}

#[function_component(ThemeSelector)]
pub fn theme_selector(props: &ThemeSelectorProps) -> Html {
    let config = use_state(|| HighlightConfig::load());
    let current_theme = config.theme.clone();
    
    let available_themes = get_available_themes();
    
    let on_theme_change = {
        let config = config.clone();
        let onchange = props.onchange.clone();
        
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let theme_name = select.value();
            
            log!("Changing theme to:", &theme_name);
            
            // Update configuration
            let mut new_config = (*config).clone();
            new_config.theme = theme_name.clone();
            new_config.save();
            config.set(new_config);
            
            // Notify parent component
            onchange.emit(theme_name);
        })
    };
    
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
    
    let highlighter = SyntaxHighlighter::with_theme(&current_theme);
    let highlighted_sample = highlighter.highlight(sample_code);
    
    html! {
        <div class="theme-selector">
            <div class="mb-4">
                <label for="theme-select" class="block font-medium mb-2">
                    {"Syntax Highlighting Theme"}
                </label>
                <select 
                    id="theme-select"
                    class="input w-full"
                    value={current_theme}
                    onchange={on_theme_change}
                >
                    {for available_themes.iter().map(|theme| {
                        html! {
                            <option value={theme.name.clone()}>
                                {theme.name.replace('_', " ").to_uppercase()}
                            </option>
                        }
                    })}
                </select>
            </div>
            
            <div class="mb-4">
                <label class="block font-medium mb-2">{"Preview"}</label>
                <div 
                    class="editor-content border rounded p-4 bg-card font-mono text-sm"
                    style="max-height: 300px; overflow-y: auto;"
                >
                    {Html::from_html_unchecked(highlighted_sample.into())}
                </div>
            </div>
            
            <div class="text-sm text-gray-500">
                <p>{"Theme changes will be applied immediately to all open files."}</p>
                <p>{"Themes are saved to your browser's local storage."}</p>
            </div>
        </div>
    }
}