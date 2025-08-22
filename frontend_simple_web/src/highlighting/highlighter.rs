// frontend_simple_web/src/highlighting/highlighter.rs
use super::config::HighlightConfig;
use super::themes::{get_theme_by_name, ColorTheme};
use std::collections::HashMap;

#[derive(PartialEq, Clone)]
enum State { 
    Code, 
    Str(char), 
    BlockComment, 
    LineComment 
}

pub struct SyntaxHighlighter {
    config: HighlightConfig,
    theme: ColorTheme,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let config = HighlightConfig::load();
        
        // Try to load custom theme first, fallback to built-in themes
        let theme = if let Some(custom_theme) = crate::components::theme_editor::CustomTheme::load_from_storage(&config.theme) {
            custom_theme.to_color_theme()
        } else {
            get_theme_by_name(&config.theme)
        };
        
        Self { config, theme }
    }
    
    pub fn with_theme(theme_name: &str) -> Self {
        let mut config = HighlightConfig::load();
        config.theme = theme_name.to_string();
        config.save();
        
        // Try to load custom theme first, fallback to built-in themes
        let theme = if let Some(custom_theme) = crate::components::theme_editor::CustomTheme::load_from_storage(theme_name) {
            custom_theme.to_color_theme()
        } else {
            get_theme_by_name(&config.theme)
        };
        
        Self { config, theme }
    }
    
    pub fn highlight(&self, src: &str) -> String {
        if !self.config.enabled {
            return format!("<span>{}</span>", esc(src));
        }
        
        // Fast path for small content
        if src.len() < self.config.fast_mode_threshold {
            return format!("<span>{}</span>", esc(src));
        }
        
        self.highlight_full(src)
    }
    
    fn highlight_full(&self, src: &str) -> String {
        let mut out = String::with_capacity(src.len() * 2);
        let mut tok = String::new();
        let mut state = State::Code;
        let mut chars = src.chars().peekable();
        
        // Create keyword lookup for faster matching
        let mut all_keywords = HashMap::new();
        
        // Add keywords from config with their classifications
        for keyword in &self.config.keywords.javascript {
            all_keywords.insert(keyword.as_str(), "keyword");
        }
        for keyword in &self.config.keywords.css {
            all_keywords.insert(keyword.as_str(), "keyword");
        }
        for keyword in &self.config.keywords.html {
            all_keywords.insert(keyword.as_str(), "tag");
        }
        for keyword in &self.config.keywords.rust {
            all_keywords.insert(keyword.as_str(), "keyword");
        }
        for keyword in &self.config.keywords.python {
            all_keywords.insert(keyword.as_str(), "keyword");
        }
        for keyword in &self.config.keywords.yaml {
            all_keywords.insert(keyword.as_str(), "keyword");
        }
        for keyword in &self.config.keywords.dockerfile {
            all_keywords.insert(keyword.as_str(), "keyword");
        }
        
        let push_tok = |out: &mut String, tok: &mut String, theme: &ColorTheme| {
            if tok.is_empty() { return; }
            
            let class = classify_enhanced(tok, &all_keywords);
            if let Some(c) = class {
                if let Some(color) = theme.colors.get(c) {
                    out.push_str(&format!(r#"<span class="{}" style="color: {}">{}</span>"#, c, color, esc(tok)));
                } else {
                    out.push_str(&format!(r#"<span class="{}">{}</span>"#, c, esc(tok)));
                }
            } else {
                out.push_str(&esc(tok));
            }
            tok.clear();
        };
        
        while let Some(ch) = chars.next() {
            match state {
                State::Code => match ch {
                    '"' | '\'' | '`' => {
                        push_tok(&mut out, &mut tok, &self.theme);
                        tok.push(ch);
                        state = State::Str(ch);
                    }
                    '/' if chars.peek() == Some(&'/') => {
                        push_tok(&mut out, &mut tok, &self.theme);
                        if let Some(color) = self.theme.colors.get("comment") {
                            out.push_str(&format!(r#"<span class="comment" style="color: {}">//"#, color));
                        } else {
                            out.push_str(r#"<span class="comment">//"#);
                        }
                        chars.next();
                        state = State::LineComment;
                    }
                    '/' if chars.peek() == Some(&'*') => {
                        push_tok(&mut out, &mut tok, &self.theme);
                        if let Some(color) = self.theme.colors.get("comment") {
                            out.push_str(&format!(r#"<span class="comment" style="color: {}"">/*"#, color));
                        } else {
                            out.push_str(r#"<span class="comment">/*"#);
                        }
                        chars.next();
                        state = State::BlockComment;
                    }
                    '#' => {
                        push_tok(&mut out, &mut tok, &self.theme);
                        if let Some(color) = self.theme.colors.get("comment") {
                            out.push_str(&format!(r#"<span class="comment" style="color: {}">#"#, color));
                        } else {
                            out.push_str(r#"<span class="comment">#"#);
                        }
                        state = State::LineComment;
                    }
                    '<' => {
                        push_tok(&mut out, &mut tok, &self.theme);
                        tok.push(ch);
                        while let Some(nc) = chars.next() {
                            tok.push(nc);
                            if nc == '>' { break; }
                        }
                        push_tok(&mut out, &mut tok, &self.theme);
                    }
                    c if c.is_whitespace() || is_punct(c) => {
                        push_tok(&mut out, &mut tok, &self.theme);
                        // Escape HTML characters
                        match c {
                            '<' => out.push_str("&lt;"),
                            '>' => out.push_str("&gt;"),
                            '&' => out.push_str("&amp;"),
                            _ => out.push(c),
                        }
                    }
                    _ => tok.push(ch),
                },
                
                State::Str(quote) => {
                    tok.push(ch);
                    if ch == quote && !tok.ends_with("\\") {
                        if let Some(color) = self.theme.colors.get("string") {
                            out.push_str(&format!(r#"<span class="string" style="color: {}">{}</span>"#, color, esc(&tok)));
                        } else {
                            out.push_str(&format!(r#"<span class="string">{}</span>"#, esc(&tok)));
                        }
                        tok.clear();
                        state = State::Code;
                    }
                }
                
                State::LineComment => {
                    // Escape HTML characters in comments
                    match ch {
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        '&' => out.push_str("&amp;"),
                        _ => out.push(ch),
                    }
                    
                    if ch == '\n' {
                        out.push_str("</span>");
                        state = State::Code;
                    }
                }
                
                State::BlockComment => {
                    // Escape HTML characters in comments
                    match ch {
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        '&' => out.push_str("&amp;"),
                        _ => out.push(ch),
                    }
                    
                    if ch == '*' && chars.peek() == Some(&'/') {
                        out.push('/');
                        chars.next();
                        out.push_str("</span>");
                        state = State::Code;
                    }
                }
            }
        }
        
        push_tok(&mut out, &mut tok, &self.theme);
        out
    }
    
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

fn classify_enhanced(tok: &str, keywords: &HashMap<&str, &str>) -> Option<&'static str> {
    let clean = tok.trim_matches(&['<','>','/','{','}','(','[',']',';',':','=','"','\'','`',','] as &[_]);
    
    // Check keywords first - convert to static str
    if let Some(&class) = keywords.get(clean) {
        // Map known classes to static strings
        return match class {
            "keyword" => Some("keyword"),
            "tag" => Some("tag"),
            _ => Some("keyword"), // fallback
        };
    }
    
    // Check for hexadecimal colors
    if clean.starts_with('#') && clean.len() >= 4 && clean.len() <= 7 {
        if clean.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
            return Some("color");
        }
    }
    
    // Check for RGB/RGBA functions
    if clean.starts_with("rgb(") || clean.starts_with("rgba(") || 
       clean.starts_with("hsl(") || clean.starts_with("hsla(") {
        return Some("color");
    }
    
    // Check for URLs
    if clean.starts_with("http://") || clean.starts_with("https://") || 
       clean.starts_with("ftp://") || clean.starts_with("file://") {
        return Some("url");
    }
    
    // Check for numbers
    if clean.parse::<f64>().is_ok() {
        return Some("number");
    }
    
    // Check for CSS units
    let units = ["px", "em", "rem", "vh", "vw", "%", "pt", "pc", "in", "cm", "mm", "deg"];
    for unit in &units {
        if clean.ends_with(unit) {
            let prefix = &clean[..clean.len() - unit.len()];
            if prefix.parse::<f64>().is_ok() {
                return Some("number");
            }
        }
    }
    
    // Check for CSS pseudo-classes and pseudo-elements
    if clean.starts_with(':') || clean.starts_with("::") {
        return Some("pseudo");
    }
    
    // Check for CSS media queries
    if clean == "@media" || clean == "@keyframes" || clean == "@import" || clean == "@font-face" {
        return Some("atrule");
    }
    
    // Check for attributes and decorators
    if clean.contains('@') && !clean.starts_with('@') {
        return Some("attr");
    }
    
    // Check for CSS selectors
    if clean.starts_with('.') && clean.len() > 1 {
        return Some("selector");
    }
    
    if clean.starts_with('#') && clean.len() > 1 && !clean.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
        return Some("selector");
    }
    
    // Check for constants (UPPERCASE)
    if clean.chars().all(|c| c.is_uppercase() || c == '_') && clean.len() > 1 {
        return Some("constant");
    }
    
    None
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

fn is_punct(c: char) -> bool {
    matches!(c, '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | ':' |
                '+' | '-' | '*' | '=' | '!' | '?' | '|' | '&' | '<' | '>')
}