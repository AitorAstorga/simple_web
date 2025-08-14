// frontend_simple_web/src/components/code_editor.rs
use gloo::console::{debug, error, log};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

use crate::api::file::{api_delete, api_move, api_upload, get_api_file, post_api_file};
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
    let file_input_ref = use_node_ref();

    /* -- load file when path changes ------------------------------------ */
    {
        let text = text.clone();
        use_effect_with(sel_path.clone(), {
            let text = text.clone();
            move |maybe_path| {
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

    /* -- Upload button & file input -------------------------------------- */
    // Trigger the hidden file picker
    let on_click_upload = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                // clear previous selection
                input.set_value("");
                input.click();
            }
        })
    };
    // Handle file(s) selected
    let on_upload = {
        let file_input_ref = file_input_ref.clone();
        let base = sel_path.clone();
        debug!("on_upload");
        Callback::from(move |_: Event| {
            let input: HtmlInputElement = file_input_ref
                .cast()
                .expect("file input should be an HtmlInputElement");
            debug!("files: {}", input.files().unwrap().length());
            if let Some(files) = input.files() {
                debug!(format!("Uploading {} files…", files.length()));
                // Log the file names
                for i in 0..files.length() {
                    let file: web_sys::File = files.item(i).unwrap();
                    debug!(file.name());
                }
                api_upload(files, base.clone());
            }
        })
    };

    html! {
        <div class="flex flex-col h-full">
            /* toolbar */
            <div class="mb-2 flex gap-2">
                <button class="btn btn-primary" onclick={on_new_folder.clone()}>{ "New Folder" }</button>
                <button class="btn btn-primary" onclick={onsave.clone()}>{ "New File" }</button>
                <button class="btn btn-primary" onclick={on_click_upload.clone()}>{ "Upload" }</button>
                {
                    if sel_path.is_some() {
                        html! {
                            <>
                                <button class="btn btn-primary" onclick={onsave.clone()}>{ "Save" }</button>
                                <button class="btn btn-secondary" onclick={onmove.clone()}>{ "Move" }</button>
                                <button class="btn btn-danger"  onclick={ondelete.clone()}>{ "Delete" }</button>
                            </>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>

            // hidden file input for upload
            <input
                type="file"
                ref={file_input_ref}
                multiple=true
                webkitdirectory=true
                style="display: none;"
                onchange={on_upload}
            />

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
                        <CodeEditorTextarea value={(*text).clone()} oninput={oninput.clone()} />
                    }
                } else {
                    html! { <h2 class="card">{"Create or select a file to start editing."}</h2> }
                }
            }
        </div>
    }
}