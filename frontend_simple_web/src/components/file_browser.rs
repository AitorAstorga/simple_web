// frontend_simple_web/src/components/file_browser.rs
use serde::{Deserialize};
use urlencoding::encode;
use wasm_bindgen_futures::spawn_local;
use yew::events::{DragEvent, MouseEvent};
use yew::prelude::*;
use gloo::console::log;
use gloo::timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;

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
    let selected_files = use_state(|| Vec::<String>::new());
    let is_drag_selecting = use_state(|| false);
    let drag_start_pos = use_state(|| (0, 0));
    let drag_current_pos = use_state(|| (0, 0));

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
        let entries = entries.clone();
        move |target_dir: String| {
            let cwd_now = cwd.clone();
            let cwd_refresh = cwd.clone();
            let entries_refresh = entries.clone();
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
                            let cwd_for_refresh = cwd_refresh.clone();
                            let entries_for_refresh = entries_refresh.clone();
                            spawn_local(async move {
                                api_move(&src_full, &dest);
                                // Refresh the directory listing after move
                                TimeoutFuture::new(500).await;
                                let path = format!("{}", encode(&cwd_for_refresh));
                                if let Ok(resp) = get_api_files(&path).await {
                                    if let Ok(list) = resp.json::<Vec<FileEntry>>().await {
                                        entries_for_refresh.set(list);
                                    }
                                }
                            });
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
            // Enhanced file type emoji icons for better recognition
            let emoji = match ext {
                &"toml" => "⚙️ ",
                &"yaml" | &"yml" => "📋 ",
                &"dockerfile" | &"Dockerfile" => "🐳 ",
                &"env" => "🔧 ",
                &"txt" | &"log" => "📄 ",
                &"xml" => "📰 ",
                &"pdf" => "📄 ",
                &"zip" | &"tar" | &"gz" => "📦 ",
                &"sh" | &"bash" => "🔧 ",
                &"py" => "🐍 ",
                &"go" => "🔷 ",
                &"java" => "☕ ",
                &"php" => "🐘 ",
                &"rb" => "💎 ",
                &"cpp" | &"c" | &"h" => "⚡ ",
                _ => "📄 ",
            };
            return html! {
                { emoji }
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
    let drop_on_dotdot = mk_drop.clone()(String::new());
    let drop_on_ul     = mk_drop.clone()((*cwd).clone());

    fn get_file_name(entry: &FileEntry) -> String {
        // Extract just the filename/dirname from the full path
        entry.path
            .rsplit('/')
            .next()
            .unwrap_or(&entry.path)
            .to_string()
    }

    /* -- bulk operations ------------------------------------------------ */
    let bulk_delete = {
        let selected_files = selected_files.clone();
        Callback::from(move |_| {
            let selected = (*selected_files).clone();
            if !selected.is_empty() {
                let count = selected.len();
                let confirm_msg = format!("Delete {} selected files/folders?", count);
                if confirm(&confirm_msg) {
                    for file_path in selected {
                        let file_to_delete = file_path.clone();
                        spawn_local(async move {
                            api_delete(&file_to_delete);
                        });
                    }
                    selected_files.set(Vec::new());
                }
            }
        })
    };
    
    let select_all = {
        let selected_files = selected_files.clone();
        let entries = entries.clone();
        let cwd = cwd.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                if input.checked() {
                    // Select all
                    let all_paths: Vec<String> = entries.iter()
                        .map(|entry| joined(&cwd, &entry.path))
                        .collect();
                    selected_files.set(all_paths);
                } else {
                    // Deselect all
                    selected_files.set(Vec::new());
                }
            }
        })
    };
    
    let selected_count = selected_files.len();
    let total_count = entries.len();
    let all_selected = selected_count > 0 && selected_count == total_count;
    
    /* -- drag selection handlers ---------------------------------------- */
    let on_mouse_down = {
        let is_drag_selecting = is_drag_selecting.clone();
        let drag_start_pos = drag_start_pos.clone();
        Callback::from(move |e: MouseEvent| {
            // Only start drag selection if clicking on empty space (not on file entries)
            if let Some(target) = e.target() {
                if let Some(element) = target.dyn_ref::<web_sys::HtmlElement>() {
                    // Check if clicked on empty space (not on interactive elements)
                    let tag_name = element.tag_name();
                    let class_name = element.class_name();
                    
                    // Don't start drag selection if clicking on interactive elements
                    if tag_name == "INPUT" || tag_name == "BUTTON" || tag_name == "LI" || 
                       tag_name == "SPAN" || tag_name == "IMG" ||
                       class_name.contains("cursor-pointer") || class_name.contains("btn") {
                        return;
                    }
                    
                    // Start drag selection for empty space (ASIDE, DIV with appropriate classes)
                    if tag_name == "ASIDE" || tag_name == "DIV" {
                        is_drag_selecting.set(true);
                        drag_start_pos.set((e.client_x(), e.client_y()));
                        e.prevent_default();
                    }
                }
            }
        })
    };

    let on_mouse_move = {
        let is_drag_selecting = is_drag_selecting.clone();
        let drag_current_pos = drag_current_pos.clone();
        Callback::from(move |e: MouseEvent| {
            if *is_drag_selecting {
                drag_current_pos.set((e.client_x(), e.client_y()));
            }
        })
    };

    let on_mouse_up = {
        let is_drag_selecting = is_drag_selecting.clone();
        let selected_files = selected_files.clone();
        let entries = entries.clone();
        let cwd = cwd.clone();
        let drag_start_pos = drag_start_pos.clone();
        let drag_current_pos = drag_current_pos.clone();
        Callback::from(move |_e: MouseEvent| {
            if *is_drag_selecting {
                // Calculate selection rectangle and select files within it
                let start = *drag_start_pos;
                let current = *drag_current_pos;
                
                let distance_x = (current.0 - start.0).abs();
                let distance_y = (current.1 - start.1).abs();
                let total_distance = ((distance_x * distance_x + distance_y * distance_y) as f64).sqrt();
                
                // Only select if the user actually dragged a significant distance (not just a click)
                if total_distance > 50.0 {
                    // For now, select all visible files when dragging
                    // In a real implementation, you'd calculate which file entries intersect with the rectangle
                    let all_paths: Vec<String> = entries.iter()
                        .map(|entry| joined(&cwd, &entry.path))
                        .collect();
                    selected_files.set(all_paths);
                } else {
                    // If it was just a click (small movement), clear selection instead
                    selected_files.set(Vec::new());
                }
                
                is_drag_selecting.set(false);
            }
        })
    };

    // Sort entries: folders first, then files, both alphabetically
    let mut sorted_entries = entries.iter().cloned().collect::<Vec<_>>();
    sorted_entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,  // folders first
            (false, true) => std::cmp::Ordering::Greater, // files second
            _ => a.path.cmp(&b.path), // same type: alphabetical
        }
    });

    /* -- render -------------------------------------------------------- */
    html! {
        <div class="file-browser-container h-full"
             onmousedown={on_mouse_down} 
             onmousemove={on_mouse_move} 
             onmouseup={on_mouse_up}>
            // File operations header
            <div class="flex items-center justify-between mb-2 pb-2 border-b file-browser-container-header">
                <div class="flex items-center gap-2">
                    <input 
                        type="checkbox" 
                        checked={all_selected} 
                        onchange={select_all}
                        title="Select all files"
                    />
                    <span class="text-sm">
                        { if selected_count > 0 {
                            format!("{} selected", selected_count)
                        } else {
                            "Files".to_string()
                        } }
                    </span>
                </div>
                { if selected_count > 0 {
                    html! {
                        <button class="btn btn-danger text-sm" onclick={bulk_delete}>
                            {"🗑️ Delete Selected"}
                        </button>
                    }
                } else {
                    html! {}
                }}
            </div>
            
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

            /* -- list entries (folders first) ------------------------------ */
            { for sorted_entries.iter().enumerate().map(|(_idx, entry)| {
                let icon_html = icon_for(&entry);
                let basename = entry.path.clone();
                let full_path = joined(&cwd, &basename);
                
                /* --------- selection checkbox --------- */
                let _selected_files_clone = selected_files.clone();
                let is_selected = selected_files.contains(&full_path);
                let on_select = {
                    let selected_files = selected_files.clone();
                    let full_path = full_path.clone();
                    Callback::from(move |e: Event| {
                        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                            let mut current_selection = (*selected_files).clone();
                            if input.checked() {
                                if !current_selection.contains(&full_path) {
                                    current_selection.push(full_path.clone());
                                }
                            } else {
                                current_selection.retain(|f| f != &full_path);
                            }
                            selected_files.set(current_selection);
                        }
                    })
                };

                /* --------- delete button (files & folders) --------- */
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
                // Both files and folders are draggable now
                let drag_cb = drag_start(basename.clone());
                
                if entry.is_dir {
                    // drop target for folders
                    let ondrop = mk_drop.clone()(joined(&cwd, &basename));

                    html! {
                        <li class="cursor-pointer hover:bg-card rounded px-1
                                flex justify-between items-center gap-2"
                            draggable="true"
                            ondragstart={drag_cb}
                            ondragover={drag_over.clone()}
                            ondrop={ondrop}
                            onclick={onclick}>
                            <div class="flex items-center gap-2">
                                <input type="checkbox" checked={is_selected} onchange={on_select} onclick={Callback::from(|e: MouseEvent| e.stop_propagation())} />
                                <span>{ icon_html }{ get_file_name(&entry) }</span>
                            </div>
                            { del_btn }
                        </li>
                    }
                } else {
                    html! {
                        <li class="cursor-pointer hover:bg-card rounded px-1
                                flex justify-between items-center gap-2"
                            draggable="true"
                            ondragstart={drag_cb}
                            onclick={onclick}>
                            <div class="flex items-center gap-2">
                                <input type="checkbox" checked={is_selected} onchange={on_select} onclick={Callback::from(|e: MouseEvent| e.stop_propagation())} />
                                <span class="flex">{ icon_html }{ get_file_name(&entry) }</span>
                            </div>
                            { del_btn }
                        </li>
                    }
                }
            }) }
        </ul>
        </div>
    }
}