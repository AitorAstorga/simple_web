// frontend_simple_web/src/highlighting/config.rs
use serde::{Deserialize, Serialize};
use gloo::storage::{LocalStorage, Storage};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HighlightConfig {
    pub theme: String,
    pub enabled: bool,
    pub fast_mode_threshold: usize,
    pub keywords: KeywordSets,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeywordSets {
    pub javascript: Vec<String>,
    pub css: Vec<String>,
    pub html: Vec<String>,
    pub rust: Vec<String>,
    pub python: Vec<String>,
    pub yaml: Vec<String>,
    pub dockerfile: Vec<String>,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            enabled: true,
            fast_mode_threshold: 100,
            keywords: KeywordSets::default(),
        }
    }
}

impl Default for KeywordSets {
    fn default() -> Self {
        Self {
            javascript: vec![
                "const", "let", "var", "function", "if", "else", "return", "for", "while",
                "async", "await", "import", "export", "class", "new", "try", "catch",
                "switch", "case", "break", "continue", "do", "typeof", "instanceof",
                "this", "super", "extends", "implements", "interface", "type", "enum"
            ].into_iter().map(String::from).collect(),
            
            css: vec![
                "display", "position", "color", "background", "border", "padding",
                "margin", "flex", "grid", "width", "height", "font", "animation",
                "transform", "opacity", "z-index", "overflow", "float", "clear",
                "text-align", "line-height", "font-size", "font-weight", "cursor"
            ].into_iter().map(String::from).collect(),
            
            html: vec![
                "html", "head", "body", "div", "span", "h1", "h2", "h3", "h4", "h5", "h6",
                "p", "ul", "ol", "li", "a", "img", "script", "style", "link", "meta",
                "title", "nav", "header", "footer", "section", "article", "aside",
                "main", "table", "tr", "td", "th", "form", "input", "button", "select"
            ].into_iter().map(String::from).collect(),
            
            rust: vec![
                "fn", "let", "mut", "const", "static", "if", "else", "match", "for", "while",
                "loop", "break", "continue", "return", "struct", "enum", "impl", "trait",
                "use", "mod", "pub", "crate", "super", "self", "where", "async", "await"
            ].into_iter().map(String::from).collect(),
            
            python: vec![
                "def", "class", "if", "elif", "else", "for", "while", "break", "continue",
                "return", "import", "from", "as", "try", "except", "finally", "with",
                "lambda", "global", "nonlocal", "async", "await", "yield"
            ].into_iter().map(String::from).collect(),
            
            yaml: vec![
                "true", "false", "null", "yes", "no", "on", "off"
            ].into_iter().map(String::from).collect(),
            
            dockerfile: vec![
                "FROM", "RUN", "CMD", "LABEL", "EXPOSE", "ENV", "ADD", "COPY",
                "ENTRYPOINT", "VOLUME", "USER", "WORKDIR", "ARG", "ONBUILD",
                "STOPSIGNAL", "HEALTHCHECK", "SHELL"
            ].into_iter().map(String::from).collect(),
        }
    }
}

const CONFIG_KEY: &str = "highlight_config";

impl HighlightConfig {
    pub fn load() -> Self {
        LocalStorage::get(CONFIG_KEY).unwrap_or_default()
    }
    
    pub fn save(&self) {
        let _ = LocalStorage::set(CONFIG_KEY, self);
    }
    
}