// frontend_simple_web/src/components/code_editor_textarea.rs
use yew::prelude::*;
use yew::virtual_dom::AttrValue;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlElement, Node, Text};
use gloo::console::log;

#[derive(Properties, PartialEq, Clone)]
pub struct CodeEditorProps {
    pub value: AttrValue,
    pub oninput: Callback<InputEvent>,
}

#[derive(Clone)]
struct CaretPosition {
    line: usize,
    column: usize,
    offset: usize,
}

#[derive(Clone)]
struct EditorState {
    content: String,
    caret_pos: CaretPosition,
    is_focused: bool,
}

#[function_component(CodeEditorTextarea)]
pub fn code_editor(props: &CodeEditorProps) -> Html {
    let editor_state = use_state(|| EditorState {
        content: props.value.to_string(),
        caret_pos: CaretPosition { line: 0, column: 0, offset: 0 },
        is_focused: false,
    });
    // Track if we should update the content (to avoid infinite loops)
    let should_update_content = use_state(|| true);
    
    // Update editor state when props change
    {
        let editor_state = editor_state.clone();
        let should_update = should_update_content.clone();
        let value = props.value.clone();
        use_effect_with(value, move |val| {
            let current_state = (*editor_state).clone();
            // Only update if the content is actually different
            if current_state.content != val.to_string() {
                should_update.set(true);
                editor_state.set(EditorState {
                    content: val.to_string(),
                    caret_pos: current_state.caret_pos,
                    is_focused: current_state.is_focused,
                });
            }
            || {}
        });
    }

    // Debounced highlighting for better performance
    let highlighted = {
        let content = editor_state.content.clone();
        let caret_pos = editor_state.caret_pos.clone();
        use_memo((content.clone(), caret_pos.line, caret_pos.column), |(val, line, column)| {
            if val.is_empty() {
                String::new()
            } else {
                let pos = CaretPosition { line: *line, column: *column, offset: 0 };
                html_highlight_with_cursor(val.as_str(), &pos)
            }
        })
    };

    /* refs for the new editor */
    let editor_ref = use_node_ref();
    let hidden_textarea_ref = use_node_ref();

    // Setup event handlers for the new contenteditable approach
    {
        let editor_ref = editor_ref.clone();
        let editor_state = editor_state.clone();
        let should_update = should_update_content.clone();
        let hidden_textarea_ref = hidden_textarea_ref.clone();

        use_effect(move || {
            // Initialize hidden textarea with current content
            if let Some(hidden_textarea) = hidden_textarea_ref.cast::<web_sys::HtmlTextAreaElement>() {
                hidden_textarea.set_value(&editor_state.content);
            }
            
            if let Some(editor) = editor_ref.cast::<HtmlElement>() {
                // Input event handler
                let _editor_clone = editor.clone();
                let state_clone = editor_state.clone();
                let should_update_clone = should_update.clone();
                let hidden_textarea_ref_clone = hidden_textarea_ref.clone();
                
                let input_cb = Closure::<dyn Fn(_)>::new(move |e: web_sys::Event| {
                    // Prevent updates during input handling
                    should_update_clone.set(false);
                    if let Some(target) = e.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                            let content = element.inner_text();
                            // Get precise caret position using pure Rust
                            let caret_pos = if let Some(offset) = get_caret_text_offset(&element) {
                                let (line, column) = offset_to_line_column(&content, offset);
                                CaretPosition { line, column, offset }
                            } else {
                                CaretPosition { line: 0, column: 0, offset: 0 }
                            };
                            
                            let current_state = (*state_clone).clone();
                            state_clone.set(EditorState {
                                content: content.clone(),
                                caret_pos,
                                is_focused: current_state.is_focused,
                            });
                            
                            // Update the hidden textarea and trigger its input event to notify parent
                            if let Some(hidden_textarea) = hidden_textarea_ref_clone.cast::<web_sys::HtmlTextAreaElement>() {
                                hidden_textarea.set_value(&content);
                                
                                // Manually trigger the input event on the hidden textarea
                                if let Some(window) = web_sys::window() {
                                    if let Some(document) = window.document() {
                                        if let Ok(event) = document.create_event("HTMLEvents") {
                                            event.init_event("input");
                                            let _ = hidden_textarea.dispatch_event(&event);
                                        }
                                    }
                                }
                            }
                        }
                    }
                });

                // Focus event handler
                let state_clone2 = editor_state.clone();
                let focus_cb = Closure::<dyn Fn(_)>::new(move |_e: web_sys::Event| {
                    let mut new_state = (*state_clone2).clone();
                    new_state.is_focused = true;
                    state_clone2.set(new_state);
                });

                // Blur event handler
                let state_clone3 = editor_state.clone();
                let blur_cb = Closure::<dyn Fn(_)>::new(move |_e: web_sys::Event| {
                    let mut new_state = (*state_clone3).clone();
                    new_state.is_focused = false;
                    state_clone3.set(new_state);
                });

                // Click handler for proper caret positioning using pure Rust
                let state_clone4 = editor_state.clone();
                let editor_clone2 = editor.clone();
                let click_cb = Closure::<dyn Fn(_)>::new(move |_e: web_sys::MouseEvent| {
                    let editor = editor_clone2.clone();
                    let state = state_clone4.clone();
                    
                    // Use setTimeout to handle click positioning after the browser processes the click
                    let timeout_cb = Closure::<dyn Fn()>::new(move || {
                        // Check if the editor element still exists before proceeding
                        if editor.parent_node().is_some() {
                            if let Some(offset) = get_caret_text_offset(&editor) {
                                let content = editor.inner_text();
                                let (line, column) = offset_to_line_column(&content, offset);
                                
                                let mut new_state = (*state).clone();
                                new_state.caret_pos = CaretPosition { line, column, offset };
                                state.set(new_state);
                            }
                        }
                    });
                    
                    if let Some(window) = web_sys::window() {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            timeout_cb.as_ref().unchecked_ref(),
                            10
                        );
                    }
                    timeout_cb.forget();
                });

                // Enhanced keydown handler with caret tracking
                let state_clone5 = editor_state.clone();
                let editor_clone3 = editor.clone();
                let keydown_cb = Closure::<dyn Fn(_)>::new(move |_e: web_sys::KeyboardEvent| {
                    let editor = editor_clone3.clone();
                    let state = state_clone5.clone();
                    
                    // Update caret position after key events
                    let timeout_cb = Closure::<dyn Fn()>::new(move || {
                        // Check if the editor element still exists before proceeding
                        if editor.parent_node().is_some() {
                            if let Some(offset) = get_caret_text_offset(&editor) {
                                let content = editor.inner_text();
                                let (line, column) = offset_to_line_column(&content, offset);
                                
                                let mut new_state = (*state).clone();
                                new_state.caret_pos = CaretPosition { line, column, offset };
                                state.set(new_state);
                            }
                        }
                    });
                    
                    if let Some(window) = web_sys::window() {
                        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                            timeout_cb.as_ref().unchecked_ref(),
                            10
                        );
                    }
                    timeout_cb.forget();
                });

                let _ = editor.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref());
                let _ = editor.add_event_listener_with_callback("focus", focus_cb.as_ref().unchecked_ref());
                let _ = editor.add_event_listener_with_callback("blur", blur_cb.as_ref().unchecked_ref());
                let _ = editor.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref());
                let _ = editor.add_event_listener_with_callback("keydown", keydown_cb.as_ref().unchecked_ref());
                
                input_cb.forget();
                focus_cb.forget();
                blur_cb.forget();
                click_cb.forget();
                keydown_cb.forget();
            }
            || {}
        });
    }

    // Effect to sync hidden textarea with editor state and trigger parent update
    {
        let hidden_textarea_ref = hidden_textarea_ref.clone();
        let content = editor_state.content.clone();
        use_effect_with(content.clone(), move |content| {
            if let Some(hidden_textarea) = hidden_textarea_ref.cast::<web_sys::HtmlTextAreaElement>() {
                let current_value = hidden_textarea.value();
                
                // Only update and trigger event if content actually changed
                if current_value != *content {
                    log!("Updating parent component with new content:", content);
                    hidden_textarea.set_value(content);
                    
                    // Trigger input event to notify parent component
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Ok(event) = document.create_event("HTMLEvents") {
                                event.init_event("input");
                                let _ = hidden_textarea.dispatch_event(&event);
                                log!("Triggered input event on hidden textarea");
                            }
                        }
                    }
                }
            }
            || {}
        });
    }

    // Effect to update content only when needed and preserve caret
    {
        let editor_ref = editor_ref.clone();
        let should_update = should_update_content.clone();
        let highlighted = highlighted.clone();
        let _editor_state = editor_state.clone();
        
        use_effect_with((should_update.clone(), highlighted.clone()), move |(should_update, highlighted)| {
            if **should_update {
                if let Some(editor) = editor_ref.cast::<HtmlElement>() {
                    // Check if editor is still in the DOM
                    if editor.parent_node().is_some() {
                        // Save current caret position
                        let current_offset = get_caret_text_offset(&editor);
                        
                        // Update content
                        editor.set_inner_html(highlighted);
                        
                        // Restore caret position if we had one (with a small delay to let DOM update)
                        if let Some(offset) = current_offset {
                            let editor_clone = editor.clone();
                            let restore_cb = Closure::<dyn Fn()>::new(move || {
                                restore_caret_position(&editor_clone, offset);
                            });
                            
                            if let Some(window) = web_sys::window() {
                                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                    restore_cb.as_ref().unchecked_ref(),
                                    0
                                );
                            }
                            restore_cb.forget();
                        }
                    }
                }
                should_update.set(false);
            }
            || {}
        });
    }

    html! {
        <div class="editor-container relative w-full flex-grow font-mono">
            <div 
                ref={editor_ref}
                class="editor-content"
                contenteditable="true"
                spellcheck="false"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                role="textbox"
                tabindex="0"
                data-gramm="false"
            >
                // Initial content will be set by the effect above
            </div>
            
            // Hidden textarea for parent component communication
            <textarea
                ref={hidden_textarea_ref}
                style="display: none;"
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

fn html_highlight_with_cursor(src: &str, _caret_pos: &CaretPosition) -> String {
    // For now, just return the regular highlighting
    // The caret will be handled by the browser's contenteditable
    html_highlight(src)
}

fn get_caret_text_offset(container: &HtmlElement) -> Option<usize> {
    // Check if container is still in the DOM
    if container.parent_node().is_none() {
        return None;
    }
    
    let window = web_sys::window()?;
    let selection = window.get_selection().ok()??;
    
    if selection.range_count() == 0 {
        return None;
    }
    
    let range = selection.get_range_at(0).ok()?;
    let start_container = range.start_container().ok()?;
    let start_offset = range.start_offset().ok()?;
    
    calculate_text_offset(&start_container, start_offset as usize, container)
}

fn calculate_text_offset(target_node: &Node, target_offset: usize, container: &HtmlElement) -> Option<usize> {
    // Check if container is still in the DOM
    if container.parent_node().is_none() {
        return None;
    }
    
    let document = web_sys::window()?.document()?;
    
    // Create a TreeWalker to traverse text nodes
    // NodeFilter::SHOW_TEXT = 4
    let walker = document
        .create_tree_walker_with_what_to_show(
            &container.clone().into(),
            4, // SHOW_TEXT constant
        )
        .ok()?;
    
    let mut total_offset = 0;
    
    // Walk through all text nodes
    while let Ok(Some(node)) = walker.next_node() {
        if let Ok(text_node) = node.clone().dyn_into::<Text>() {
            if node == *target_node {
                // Found the target node, add the offset within this node
                let text_content = text_node.text_content().unwrap_or_default();
                total_offset += target_offset.min(text_content.len());
                break;
            } else {
                // Add the entire length of this text node
                total_offset += text_node.text_content().unwrap_or_default().len();
            }
        }
    }
    
    Some(total_offset)
}


fn offset_to_line_column(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut column = 0;
    
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        
        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    
    (line, column)
}

fn restore_caret_position(container: &HtmlElement, target_offset: usize) {
    // Check if container is still in the DOM
    if container.parent_node().is_none() {
        return;
    }
    
    if let Some(window) = web_sys::window() {
        if let Ok(Some(selection)) = window.get_selection() {
            if let Some(document) = window.document() {
                if let Ok(range) = document.create_range() {
                    // Find the text node and offset that corresponds to our target offset
                    if let Some((node, offset)) = find_text_node_at_offset(container, target_offset) {
                        if range.set_start(&node, offset as u32).is_ok() {
                            range.collapse_with_to_start(true);
                            let _ = selection.remove_all_ranges();
                            let _ = selection.add_range(&range);
                        }
                    }
                }
            }
        }
    }
}

fn find_text_node_at_offset(container: &HtmlElement, target_offset: usize) -> Option<(Node, usize)> {
    // Check if container is still in the DOM
    if container.parent_node().is_none() {
        return None;
    }
    
    if let Some(document) = web_sys::window()?.document() {
        if let Ok(walker) = document.create_tree_walker_with_what_to_show(
            &container.clone().into(),
            4, // SHOW_TEXT constant
        ) {
            let mut current_offset = 0;
            
            while let Ok(Some(node)) = walker.next_node() {
                if let Ok(text_node) = node.clone().dyn_into::<Text>() {
                    let text_content = text_node.text_content().unwrap_or_default();
                    let text_length = text_content.len();
                    
                    if current_offset + text_length >= target_offset {
                        // Found the node containing our target offset
                        let offset_in_node = target_offset.saturating_sub(current_offset);
                        return Some((node, offset_in_node));
                    }
                    
                    current_offset += text_length;
                }
            }
        }
    }
    None
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