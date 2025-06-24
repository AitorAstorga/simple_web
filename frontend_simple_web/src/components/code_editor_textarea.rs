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
    let highlighted = use_memo(props.value.clone(), |val| {
        html_highlight(val.as_str())
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
                    pre_clone.set_scroll_top(ta_clone.scroll_top());
                    pre_clone.set_scroll_left(ta_clone.scroll_left());
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

    const KW_JS  : &[&str] = &["const","let","function","if","else","return","for","while",
                               "async","await","import","export","class","new","try","catch",
                               "switch","case","break"];
    const KW_CSS : &[&str] = &["display","position","color","background","border","padding",
                               "margin","flex","grid","width","height","font","animation",
                               "transform"];
    const KW_HTML: &[&str] = &["html","head","body","div","span","h1","h2","h3","p","ul","li",
                               "script","style","link"];

    let mut out   = String::with_capacity(src.len() * 2);
    let mut tok   = String::new();
    let mut state = State::Code;
    let mut it    = src.chars().peekable();

    /* push the current token with an optional <span class=""> wrapper */
    let push_tok = |out: &mut String, tok: &mut String| {
        if tok.is_empty() { return; }
        let class = classify(tok, KW_JS, KW_CSS, KW_HTML);
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
                '\'' | '"' | '`' => { push_tok(&mut out, &mut tok); tok.push(ch); state = State::Str(ch); }
                '/' if it.peek() == Some(&'/') => { push_tok(&mut out, &mut tok); tok.push_str("//"); it.next(); state = State::LineComment; }
                '/' if it.peek() == Some(&'*') => { push_tok(&mut out, &mut tok); tok.push_str("/*"); it.next(); state = State::BlockComment; }
                '<' => {
                    push_tok(&mut out, &mut tok);
                    tok.push(ch);
                    while let Some(nc) = it.next() { tok.push(nc); if nc == '>' { break; } }
                    push_tok(&mut out, &mut tok);
                }
                c if c.is_whitespace() || is_punct(c) => { push_tok(&mut out, &mut tok); out.push(c); }
                _ => tok.push(ch),
            },

            /* ───────────── string literal ───────────── */
            State::Str(q) => {
                tok.push(ch);
                if ch == q && !tok.ends_with("\\") { push_tok(&mut out, &mut tok); state = State::Code; }
            }

            /* ───────────── // line comment ───────────── */
            State::LineComment => {
                tok.push(ch);
                if ch == '\n' { push_tok(&mut out, &mut tok); state = State::Code; }
            }

            /* ───────────── /* block comment */ ───────────── */
            State::BlockComment => {
                tok.push(ch);
                if ch == '*' && it.peek() == Some(&'/') {
                    tok.push('/'); it.next();
                    push_tok(&mut out, &mut tok); state = State::Code;
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

fn classify(tok: &str, js:&[&str], css:&[&str], html:&[&str]) -> Option<&'static str> {
    let clean = tok.trim_matches(&['<','>','/','{','}','(','[',']',';'] as &[_]);
    if js.contains(&clean)           { Some("kw")   }
    else if css.contains(&clean)     { Some("kw")   }
    else if html.contains(&clean)    { Some("tag")  }
    else                             { None         }
}

fn is_punct(c: char) -> bool {
    matches!(c,'(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '.' | ':' |
                '+' | '-' | '*' | '=' | '!' | '?' | '|' | '&' | '<' | '>')
}
