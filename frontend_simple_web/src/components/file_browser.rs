// frontend_simple_web/src/components/file_browser.rs
use serde::{Deserialize};
use urlencoding::encode;
use wasm_bindgen_futures::spawn_local;
use yew::events::DragEvent;
use yew::prelude::*;
use gloo::console::log;

use crate::api::file::{api_delete, get_api_files, api_move};

#[derive(Deserialize, Clone, PartialEq)]
pub struct FileEntry {
    pub path: String,
    pub is_dir: bool,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_select: Callback<String>,
}

#[function_component(FileBrowser)]
pub fn file_browser(props: &Props) -> Html {
    /* -- state -------------------------------------------------------- */
    let cwd     = use_state(|| String::new());
    let entries = use_state(|| Vec::<FileEntry>::new());

    /* -- fetch dir listing -------------------------------------------- */
    {
        let cwd = cwd.clone();
        let entries = entries.clone();
        use_effect_with(cwd.clone(), move |dir| {
            let entries = entries.clone();
            let path = format!("{}", encode(dir));

            spawn_local(async move {
                if let Ok(resp) = get_api_files(&path).await
                {
                    if let Ok(list) = resp.json::<Vec<FileEntry>>().await {
                        entries.set(list);
                    }
                }
            });
            || ()
        });
    }

    /* -- helpers ------------------------------------------------------- */
    fn joined(dir: &str, leaf: &str) -> String {
        if dir.is_empty() { leaf.into() } else { format!("{dir}/{leaf}") }
    }
    fn confirm(msg: &str) -> bool {
        web_sys::window().map(|w| w.confirm_with_message(msg).unwrap_or(false)).unwrap_or(false)
    }

    /* -- click to open / select --------------------------------------- */
    let choose = props.on_select.clone();
    let cwd_click = cwd.clone();
    let click_entry = Callback::from(move |ent: FileEntry| {
        if ent.is_dir {
            cwd_click.set(ent.path);
        } else {
            choose.emit(ent.path.clone());
        }
    });

    /* -- drag‐start ---------------------------------------------------- */
    let drag_start = {
        let cwd = cwd.clone();
        move |name: String| {
            let src = joined(&cwd, &name);
            Callback::from(move |e: DragEvent| {
                e.data_transfer().and_then(|dt| dt.set_data("text/plain", &src).ok());
            })
        }
    };

    /* -- drop target factory ------------------------------------------ */
    let mk_drop = {
        let cwd = cwd.clone();
        move |target_dir: String| {
            let cwd_now = cwd.clone();
            Callback::from(move |e: DragEvent| {
                e.prevent_default();
                if let Some(dt) = e.data_transfer() {
                    if let Ok(src_full) = dt.get_data("text/plain") {
                        let leaf = src_full.rsplit('/').next().unwrap_or(&src_full);
                        let dest_dir = if target_dir.is_empty() {
                            (*cwd_now).clone()
                        } else {
                            target_dir.clone()
                        };
                        let dest = joined(&dest_dir, leaf);
                        if dest != src_full {
                            log!(&format!("move {src_full} to {dest}"));
                            api_move(&src_full, &dest);
                        }
                    }
                }
            })
        }
    };
    let drag_over = Callback::from(|e: DragEvent| e.prevent_default());

    /* -- icon -------------------------------------------------------- */
    fn icon_for(entry: &FileEntry) -> Html {
        if entry.is_dir {
            return html! {
                { "📁 " }
            };
        }

        let ext = &entry.path.rsplit('.').next().unwrap_or("");

        let svg = match ext {
            &"rs"   => "rust.svg",
            &"html" | &"htm" => "html.svg",
            &"css"  => "css.svg",
            &"js"   => "js.svg",
            &"ts"   => "ts.svg",
            &"json" => "json.svg",
            &"md"   => "markdown.svg",
            &"png" | &"jpg" | &"jpeg" | &"gif" | &"svg" => "image.svg",
            _      => "",
        };

        if svg.is_empty() {
            return html! {
                { "📄 " }
            };
        }

        html! {
            <img class="icon"
                src={format!("/static/logos/{svg}")}
                alt={format!("{ext} icon")}/>
        }
    }

    /* -- “..” up nav --------------------------------------------------- */
    let up = {
        let cwd = cwd.clone();
        Callback::from(move |_| {
            if let Some(i) = cwd.rfind('/') { cwd.set(cwd[..i].to_string()) }
            else { cwd.set(String::new()) }
        })
    };
    let drop_on_dotdot = mk_drop(String::new());
    let drop_on_ul     = mk_drop((*cwd).clone());

    fn get_file_name(entry: &FileEntry) -> String {
        entry.path.clone().trim_start_matches('/').into()
    }

    /* -- render -------------------------------------------------------- */
    html! {
        <ul class="space-y-1" ondragover={drag_over.clone()} ondrop={drop_on_ul}>
            { if !cwd.is_empty() {
                    html! {
                        <li class="cursor-pointer hover:bg-card rounded px-1"
                            onclick={up}
                        ondragover={drag_over.clone()}
                            ondrop={drop_on_dotdot.clone()}>
                            { "📁 .." }
                        </li>
                    }
            } else { html!{} } }

            /* -- list entries----------------------------------------------- */
            { for entries.iter().cloned().map(|entry| {
                let icon_html = icon_for(&entry);
                let basename = entry.path.clone();

                /* --------- delete button (files & folders) --------- */
                let full_path = joined(&cwd, &basename);
                let confirm_msg = if entry.is_dir {
                    format!("Delete folder “{}” and its contents?", full_path)
                        } else {
                    format!("Delete file “{}”?", full_path)
                };
                let del_cb = {
                    let fp = full_path.clone();
                    Callback::from(move |_| {
                        if confirm(&confirm_msg) {
                            let file_to_delete = fp.clone();
                            spawn_local(async move {
                                api_delete(&file_to_delete);
                            });
                        }
                    })
                };
                let del_btn = html! { <button class="text-red-600" onclick={del_cb}>{"🗑"}</button> };

                /* --------- entry-specific UI --------- */
                // click handler
                let ent_for_click = entry.clone();
                let onclick = click_entry.reform(move |_| ent_for_click.clone());
                if entry.is_dir {
                    // drop target
                    let ondrop = mk_drop(joined(&cwd, &basename));

                    html! {
                        <li class="cursor-pointer hover:bg-card rounded px-1
                                flex justify-between items-center gap-2"
                            onclick={onclick}
                            ondragover={drag_over.clone()}
                            ondrop={ondrop}>
                            <span>{ icon_html }{ &entry.path }</span>
                            { del_btn }
                        </li>
                    }
                } else {
                    // drag handler
                    let drag_cb = drag_start(basename.clone());

                    html! {
                            <li class="cursor-pointer hover:bg-card rounded px-1
                                    flex justify-between items-center gap-2"
                                onclick={onclick}
                            draggable="true"
                                ondragstart={drag_cb}>
                                <span class="flex">{ icon_html }{ get_file_name(&entry) }</span>
                                { del_btn }
                        </li>
                    }
                }
            }) }
        </ul>
    }
}