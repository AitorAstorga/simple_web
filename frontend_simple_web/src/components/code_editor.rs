// frontend_simple_web/src/components/code_editor.rs
use gloo::{console::error, net::http::Request};
use serde_json::json;
use urlencoding::encode;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

const AUTH: &str = "secret123";

#[derive(Properties, PartialEq)]
pub struct Props {
    pub path: Option<String>,          // selected file (relative to ROOT)
}

#[function_component(CodeEditor)]
pub fn code_editor(props: &Props) -> Html {
    /* -- state ---------------------------------------------------------- */
    let text     = use_state(|| String::new());
    let sel_path = props.path.clone();

    /* -- load file when path changes ------------------------------------ */
    {
        let text = text.clone();
        use_effect_with(sel_path.clone(), move |maybe| {
            if let Some(p) = maybe {
                let url = format!("/api/file?path={}", encode(p));
                let text = text.clone();
                spawn_local(async move {
                    if let Ok(resp) = Request::get(&url).header("Authorization", AUTH).send().await {
                        match resp.text().await {
                            Ok(body) => text.set(body),
                            Err(e)   => error!(format!("body err: {e:?}")),
                        }
                    }
                });
            } else {
                text.set(String::new());
            }
            || ()
        });
    }

    /* -- thin async helpers --------------------------------------------- */
    async fn api_post(path: &str, content: &str) {
        let body = json!({ "path": path, "content": content });
        let _ = Request::post("/api/file")
            .header("Authorization", AUTH)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .expect("req").send().await;
    }
    async fn api_delete(path: &str) {
        let url = format!("/api/file?path={}", encode(path));
        let _ = Request::delete(&url).header("Authorization", AUTH).send().await;
    }
    async fn api_move(from: &str, to: &str) {
        let body = json!({ "from": from, "to": to });
        let _ = Request::post("/api/move")
            .header("Authorization", AUTH)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body).unwrap())
            .expect("req").send().await;
    }
    fn prompt(msg: &str)  -> Option<String> { web_sys::window()?.prompt_with_message(msg).ok()? }
    fn confirm(msg: &str) -> bool { web_sys::window().map(|w| w.confirm_with_message(msg).unwrap_or(false)).unwrap_or(false) }
    fn reload() { let _ = web_sys::window().map(|w| w.location().reload()); }

    /* -- textarea on-input ---------------------------------------------- */
    let oninput = {
        let text = text.clone();
        Callback::from(move |e: InputEvent| {
            let el: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            text.set(el.value());
        })
    };

    /* -- Save / New File button ----------------------------------------- */
    let onsave = {
        let path_opt  = sel_path.clone();
        let content   = (*text).clone();
        Callback::from(move |_| {
            let p_opt = path_opt.clone();
            let c     = content.clone();
            spawn_local(async move {
                match p_opt {
                    Some(p) => api_post(&p, &c).await, // overwrite existing
                    None    => {
                        if let Some(new_p) = prompt("New file path (e.g. js/app.js)") {
                            api_post(&new_p, &c).await; // create new
                        }
                    }
                }
                reload();
            });
        })
    };

    /* -- Delete button -------------------------------------------------- */
    let ondelete = {
        let path_opt = sel_path.clone();
        Callback::from(move |_| {
            if let Some(p) = &path_opt {
                let p = p.clone();
                if confirm(&format!("Delete {p}?")) {
                    spawn_local(async move { api_delete(&p).await; reload(); });
                }
            }
        })
    };

    /* -- Move/Rename button --------------------------------------------- */
    let onmove = {
        let path_opt = sel_path.clone();
        Callback::from(move |_| {
            if let Some(from) = &path_opt {
                if let Some(to) = prompt("New path (relative to root)") {
                    let from = from.clone();
                    spawn_local(async move { api_move(&from, &to).await; reload(); });
                }
            }
        })
    };

    /* -- New Folder button ----------------------------------------------- */
    let on_new_folder = Callback::from(move |_| {
        if let Some(folder) = prompt("Folder name (e.g. img/icons)") {
            let tmp = format!("{folder}/__tmp__"); // write + delete tmp file
            spawn_local(async move {
                api_post(&tmp, "").await;
                api_delete(&tmp).await;
                reload();
            });
        }
    });

    /* -- UI RENDER ------------------------------------------------------ */
    html! {
        <div class="flex flex-col h-full">
            /* toolbar */
            <div class="mb-2 flex gap-2">
                <button class="btn btn-primary" onclick={on_new_folder}>{"New Folder"}</button>
                <button class="btn btn-primary" onclick={onsave.clone()}>
                    { if sel_path.is_some() { "Save" } else { "New File" } }
                </button>
                { if sel_path.is_some() {
                    html! {
                        <>
                            <button class="btn btn-secondary" onclick={onmove}>{"Move"}</button>
                            <button class="btn btn-danger"  onclick={ondelete}>{"Delete"}</button>
                        </>
                    }
                } else { html!{} } }
            </div>

            /* filename */
            { if sel_path.is_some() {
                html! { <strong id="filename">{ sel_path.clone().unwrap() }</strong> }
            } else {
                html! {  }
            }}

            /* editor pane */
            { if sel_path.is_some() {
                html! { <textarea class="flex-grow font-mono w-full border rounded p-2"
                                  value={(*text).clone()} oninput={oninput}/> }
            } else {
                html! { <h2 class="card">{"Create or select a file to start editing."}</h2> }
            }}
        </div>
    }
}
