// frontend_simple_web/src/components/code_editor_textarea.rs
use yew::prelude::*;
use yew::virtual_dom::AttrValue;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::HtmlElement;
use crate::highlighting::highlighter::SyntaxHighlighter;

#[derive(Properties, PartialEq, Clone)]
pub struct CodeEditorProps {
    pub value: AttrValue,
    pub oninput: Callback<InputEvent>,
}

#[function_component(CodeEditorTextarea)]
pub fn code_editor(props: &CodeEditorProps) -> Html {
    let editor_ref = use_node_ref();
    let hidden_textarea_ref = use_node_ref();
    
    // Simple debounce flag to prevent update loops
    let updating = use_state(|| false);

    // Initialize and update editor content from props
    {
        let editor_ref = editor_ref.clone();
        let updating = updating.clone();
        let value = props.value.clone();
        
        use_effect_with(value, move |val| {
            if !*updating {
                if let Some(editor) = editor_ref.cast::<HtmlElement>() {
                    if editor.parent_node().is_some() {
                        let current_content = editor.inner_text();
                        if current_content != val.as_str() {
                            // Apply syntax highlighting only on external updates
                            let highlighted = if val.is_empty() {
                                String::from("<span></span>")
                            } else {
                                let highlighter = SyntaxHighlighter::new();
                                highlighter.highlight(val.as_str())
                            };
                            editor.set_inner_html(&highlighted);
                        }
                    }
                }
            }
            || {}
        });
    }

    // Setup minimal event handlers
    {
        let editor_ref = editor_ref.clone();
        let hidden_textarea_ref = hidden_textarea_ref.clone();
        let updating = updating.clone();

        use_effect(move || {
            if let Some(editor) = editor_ref.cast::<HtmlElement>() {
                // Initialize empty content
                if editor.inner_html().is_empty() {
                    editor.set_inner_html("<span></span>");
                }

                // Extremely optimized input handler - minimal work
                let hidden_textarea_ref_clone = hidden_textarea_ref.clone();
                let updating_clone = updating.clone();
                
                let input_cb = Closure::<dyn Fn(_)>::new(move |e: web_sys::Event| {
                    // Set updating flag to prevent props updates
                    updating_clone.set(true);
                    
                    // Get content directly from event target - no DOM queries
                    if let Some(target) = e.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::HtmlElement>() {
                            let content = element.inner_text();
                            
                            // Direct hidden textarea update - no events, no delays
                            if let Some(hidden_textarea) = hidden_textarea_ref_clone.cast::<web_sys::HtmlTextAreaElement>() {
                                hidden_textarea.set_value(&content);
                                
                                // Trigger parent callback with minimal overhead
                                if let Some(window) = web_sys::window() {
                                    if let Some(document) = window.document() {
                                        if let Ok(input_event) = document.create_event("HTMLEvents") {
                                            input_event.init_event("input");
                                            let _ = hidden_textarea.dispatch_event(&input_event);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Reset updating flag immediately - no timeout needed
                    updating_clone.set(false);
                });

                // Minimal click handler - no caret tracking
                let click_cb = Closure::<dyn Fn(_)>::new(move |_e: web_sys::MouseEvent| {
                    // Let browser handle click positioning naturally
                    // No manual caret manipulation needed
                });

                // Add event listeners
                let _ = editor.add_event_listener_with_callback("input", input_cb.as_ref().unchecked_ref());
                let _ = editor.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref());
                
                // Prevent memory leaks
                input_cb.forget();
                click_cb.forget();
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
                // Content will be set by effects
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

