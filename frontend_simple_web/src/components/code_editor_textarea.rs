// frontend_simple_web/src/components/code_editor_textarea.rs
use yew::prelude::*;
use yew::virtual_dom::AttrValue;
use wasm_bindgen::{closure::Closure, JsCast};

#[derive(Properties, PartialEq, Clone)]
pub struct CodeEditorProps {
    pub value: AttrValue,
    pub oninput: Callback<InputEvent>,
}

#[function_component(CodeEditorTextarea)]
pub fn code_editor(props: &CodeEditorProps) -> Html {
    // Debounced highlighting for better performance
    let highlighted = use_memo(props.value.clone(), |val| {
        // Only re-highlight if content has actually changed
        if val.is_empty() {
            String::new()
        } else {
            html_highlight(val.as_str())
        }
    });

    /* refs for scroll synchronisation */
    let textarea_ref = use_node_ref();
    let pre_ref      = use_node_ref();

    {
        let textarea_ref = textarea_ref.clone();
        let pre_ref      = pre_ref.clone();

        /* one-time effect to wire the scroll listener */
        use_effect(move || {
            if let (Some(ta), Some(pre)) = (
                textarea_ref.cast::<web_sys::HtmlTextAreaElement>(),
                pre_ref.cast::<web_sys::HtmlElement>(),
            ) {
                // ── make clones the closure can own ───────────────────────────────
                let ta_clone  = ta.clone();
                let pre_clone = pre.clone();

                let cb = Closure::<dyn Fn(_)>::new(move |_e: web_sys::Event| {
                    // Enhanced scroll synchronization with better precision
                    let scroll_top = ta_clone.scroll_top();
                    let scroll_left = ta_clone.scroll_left();
                    
                    // Synchronize scrolling with exact pixel positioning
                    pre_clone.set_scroll_top(scroll_top);
                    pre_clone.set_scroll_left(scroll_left);
                });

                // the original `ta` is still available for the registration call
                let _ = ta.add_event_listener_with_callback("scroll", cb.as_ref().unchecked_ref());
                cb.forget();          // keep the callback alive
            }
            || {}
        });
    }

    html! {
        <div class="editor-container relative w-full flex-grow font-mono">
            <pre ref={pre_ref} class="editor-highlight">
                { Html::from_html_unchecked((*highlighted).clone().into()) }
            </pre>

            <textarea
                ref={textarea_ref}
                class="editor-input"
                value={props.value.clone()}
                oninput={props.oninput.clone()}
            />
        </div>
    }
}

fn html_highlight(src: &str) -> String {
    #[derive(PartialEq)]
    enum State { Code, Str(char), BlockComment, LineComment }

    // JavaScript/TypeScript keywords
    const KW_JS: &[&str] = &["const","let","var","function","if","else","return","for","while",
                             "async","await","import","export","class","new","try","catch",
                             "switch","case","break","continue","do","typeof","instanceof",
                             "this","super","extends","implements","interface","type","enum"];
    
    // CSS properties and values
    const KW_CSS: &[&str] = &["display","position","color","background","border","padding",
                              "margin","flex","grid","width","height","font","animation",
                              "transform","opacity","z-index","overflow","float","clear",
                              "text-align","line-height","font-size","font-weight","cursor"];
    
    // HTML tags
    const KW_HTML: &[&str] = &["html","head","body","div","span","h1","h2","h3","h4","h5","h6",
                               "p","ul","ol","li","a","img","script","style","link","meta",
                               "title","nav","header","footer","section","article","aside",
                               "main","table","tr","td","th","form","input","button","select"];
    
    // YAML keywords
    const KW_YAML: &[&str] = &["true","false","null","yes","no","on","off"];
    
    // TOML keywords  
    const KW_TOML: &[&str] = &["true","false"];
    
    // Dockerfile keywords
    const KW_DOCKERFILE: &[&str] = &["FROM","RUN","CMD","LABEL","EXPOSE","ENV","ADD","COPY",
                                     "ENTRYPOINT","VOLUME","USER","WORKDIR","ARG","ONBUILD",
                                     "STOPSIGNAL","HEALTHCHECK","SHELL"];

    let mut out   = String::with_capacity(src.len() * 2);
    let mut tok   = String::new();
    let mut state = State::Code;
    let mut it    = src.chars().peekable();

    /* push the current token with an optional <span class=""> wrapper */
    let push_tok = |out: &mut String, tok: &mut String| {
        if tok.is_empty() { return; }
        let class = classify_enhanced(tok, KW_JS, KW_CSS, KW_HTML, KW_YAML, KW_TOML, KW_DOCKERFILE);
        if let Some(c) = class {
            out.push_str(&format!(r#"<span class="{c}">{}</span>"#, esc(tok)));
        } else {
            out.push_str(&esc(tok));
        }
        tok.clear();
    };

    while let Some(ch) = it.next() {
        match state {
            /* ───────────── ordinary code ───────────── */
            State::Code => match ch {
                '\'' | '"' | '`' => { 
                    push_tok(&mut out, &mut tok); 
                    tok.push(ch); 
                    state = State::Str(ch); 
                }
                '/' if it.peek() == Some(&'/') => { 
                    push_tok(&mut out, &mut tok); 
                    out.push_str(r#"<span class="comment">//"#);
                    it.next(); 
                    state = State::LineComment; 
                }
                '/' if it.peek() == Some(&'*') => { 
                    push_tok(&mut out, &mut tok); 
                    out.push_str(r#"<span class="comment">/*"#);
                    it.next(); 
                    state = State::BlockComment; 
                }
                '#' => {
                    push_tok(&mut out, &mut tok);
                    out.push_str(r#"<span class="comment">#"#);
                    state = State::LineComment;
                }
                '<' => {
                    push_tok(&mut out, &mut tok);
                    tok.push(ch);
                    while let Some(nc) = it.next() { 
                        tok.push(nc); 
                        if nc == '>' { break; } 
                    }
                    push_tok(&mut out, &mut tok);
                }
                c if c.is_whitespace() || is_punct(c) => { 
                    push_tok(&mut out, &mut tok); 
                    out.push(c); 
                }
                _ => tok.push(ch),
            },

            /* ───────────── string literal ───────────── */
            State::Str(q) => {
                tok.push(ch);
                if ch == q && !tok.ends_with("\\") { 
                    out.push_str(&format!(r#"<span class="string">{}</span>"#, esc(&tok)));
                    tok.clear();
                    state = State::Code; 
                }
            }

            /* ───────────── // line comment ───────────── */
            State::LineComment => {
                out.push(ch);
                if ch == '\n' { 
                    out.push_str("</span>");
                    state = State::Code; 
                }
            }

            /* ───────────── /* block comment */ ───────────── */
            State::BlockComment => {
                out.push(ch);
                if ch == '*' && it.peek() == Some(&'/') {
                    out.push('/'); 
                    it.next();
                    out.push_str("</span>");
                    state = State::Code;
                }
            }
        }
    }
    push_tok(&mut out, &mut tok);   // flush tail
    out
}

/* ---------- helpers -------------------------------------------------------------------- */
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

fn classify_enhanced(tok: &str, js: &[&str], css: &[&str], html: &[&str], yaml: &[&str], toml: &[&str], dockerfile: &[&str]) -> Option<&'static str> {
    let clean = tok.trim_matches(&['<','>','/','{','}','(','[',']',';',':','=','"','\'','`',','] as &[_]);
    
    // Check for hexadecimal colors - Unicode safe
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
    
    // Check for numbers (including units) - Unicode safe
    if clean.parse::<f64>().is_ok() {
        return Some("number");
    }
    
    // Check for CSS units (px, em, etc.) - Unicode safe
    let units = ["px", "em", "rem", "vh", "vw", "%", "pt", "pc", "in", "cm", "mm"];
    for unit in &units {
        if clean.ends_with(unit) {
            let prefix = &clean[..clean.len() - unit.len()];
            if prefix.parse::<f64>().is_ok() {
                return Some("number");
            }
        }
    }
    
    // Check for degrees - Unicode safe
    if clean.ends_with("deg") {
        let prefix = &clean[..clean.len() - 3];
        if prefix.parse::<f64>().is_ok() {
            return Some("number");
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
    
    // Check for various keywords
    if js.contains(&clean) || css.contains(&clean) || yaml.contains(&clean) || 
       toml.contains(&clean) || dockerfile.contains(&clean) {
        Some("keyword")
    } else if html.contains(&clean) {
        Some("tag")
    } else if clean.starts_with('#') && clean.len() > 1 {
        Some("comment")
    } else if clean.contains('@') && !clean.starts_with('@') {
        Some("attr")  // decorators, email addresses, etc.
    } else if clean.starts_with('.') && clean.len() > 1 {
        Some("selector")  // CSS classes
    } else if clean.starts_with('#') && clean.len() > 1 && !clean.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
        Some("selector")  // CSS IDs (not hex colors)
    } else if clean.chars().all(|c| c.is_uppercase() || c == '_') && clean.len() > 1 {
        Some("constant")  // CONSTANTS
    } else {
        None
    }
}

fn is_punct(c: char) -> bool {
    matches!(c,'(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | ':' |
                '+' | '-' | '*' | '=' | '!' | '?' | '|' | '&' | '<' | '>')
}