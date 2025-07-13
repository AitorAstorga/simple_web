// frontend_simple_web/src/components/code_editor.rs
use gloo::console::error;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::file::{api_move, get_api_file, post_api_file, api_delete};
use crate::components::code_editor_textarea::CodeEditorTextarea;

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
        use_effect_with(sel_path.clone(), {
            let text = text.clone();
            move |maybe_path| {
                // pull an owned PathBuf out of the Option, rather than borrowing
                if let Some(path) = maybe_path.clone() {
                    let text = text.clone();
                    spawn_local(async move {
                        if let Ok(resp) = get_api_file(&path).await {
                            match resp.text().await {
                                Ok(body) => text.set(body),
                                Err(e)   => error!(format!("body err: {:?}", e)),
                            }
                        }
                    });
                } else {
                    text.set(String::new());
                }
                move || {}
            }
        });

    }

    /* -- textarea on-input ---------------------------------------------- */
    let oninput = {
        let text = text.clone();
        Callback::from(move |e: InputEvent| {
            let txt: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            text.set(txt.value());
        })
    };

    /* -- Save / New File button ----------------------------------------- */
    let onsave = {
        let sel_path = sel_path.clone();
        let text = (*text).clone();
        Callback::from(move |_| {
            if let Some(path) = &sel_path {
                post_api_file(path.clone(), text.clone());
            } else if let Some(new_p) = web_sys::window()
                .unwrap()
                .prompt_with_message("New file path (e.g. js/app.js)")
                .unwrap()
            {
                post_api_file(new_p, text.clone());
            }
        })
    };

    /* -- Delete button -------------------------------------------------- */
    let ondelete = {
        let api_delete = api_delete.clone();
        let sel_path   = sel_path.clone();
        Callback::from(move |_| {
            if let Some(path) = &sel_path {
                if web_sys::window()
                    .unwrap()
                    .confirm_with_message(&format!("Delete {}?", path))
                    .unwrap()
                {
                    api_delete(path.clone());
                }
            }
        })
    };

    /* -- Move/Rename button --------------------------------------------- */
    let onmove = {
        let api_move = api_move.clone();
        let sel_path = sel_path.clone();
        Callback::from(move |_| {
            if let Some(from) = &sel_path {
                if let Some(to) = web_sys::window()
                    .unwrap()
                    .prompt_with_message("New path (relative to root)")
                    .unwrap()
                {
                    api_move(from.clone(), to);
                }
            }
        })
    };

    /* -- New Folder button ----------------------------------------------- */
    let on_new_folder = {
        let api_delete = api_delete.clone();
        Callback::from(move |_| {
            if let Some(folder) = web_sys::window()
                .unwrap()
                .prompt_with_message("Folder name (e.g. img/icons)")
                .unwrap()
            {
                let tmp = format!("{}/__tmp__", folder);
                post_api_file(tmp.clone(), String::new());
                api_delete(tmp);
            }
        })
    };

    html! {
        <div class="flex flex-col h-full">
            /* toolbar */
            <div class="mb-2 flex gap-2">
                <button class="btn btn-primary" onclick={on_new_folder.clone()}>{"New Folder"}</button>
                <button class="btn btn-primary" onclick={onsave.clone()}>{ "New File" }</button>
                { 
                    if sel_path.is_some() { 
                        html! {
                            <button class="btn btn-primary" onclick={onsave.clone()}>{"Save" }</button>
                        }
                    } else {
                        html!{}
                    }
                }
                {
                    if sel_path.is_some() {
                        html! {
                            <>
                                <button class="btn btn-secondary" onclick={onmove.clone()}>{"Move"}</button>
                                <button class="btn btn-danger"  onclick={ondelete.clone()}>{"Delete"}</button>
                            </>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>

            /* filename */
            {
                if let Some(ref p) = sel_path {
                    html! { <strong id="filename">{ p.clone() }</strong> }
                } else {
                    html!{}
                }
            }

            /* editor pane */
            {
                if sel_path.is_some() {
                    html! {
                        <CodeEditorTextarea
                            value={(*text).clone()}
                            oninput={oninput.clone()} />
                    }
                } else {
                    html! { <h2 class="card">{"Create or select a file to start editing."}</h2> }
                }
            }
        </div>
    }
}