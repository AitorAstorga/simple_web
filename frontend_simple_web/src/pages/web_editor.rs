// frontend_simple_web/src/pages/web_editor.rs
use yew::prelude::*;
use crate::components::{file_browser::FileBrowser, code_editor::CodeEditor};

#[function_component(WebEditor)]
pub fn web_editor() -> Html {
    let selected = use_state(|| None as Option<String>);

    let on_select = {
        let selected = selected.clone();
        Callback::from(move |p: String| selected.set(Some(p)))
    };

    html! {
        <div>
            <header>
                <img src="static/img/aichan.svg" alt="aichan" class="logo" />
                <h1 class="font-bold">{ "aichan's Simple Web - Web Editor" }</h1>
            </header>
            <div class="grid grid-cols-4 h-screen">
                <aside class="col-span-1 p-3 border-r overflow-y-auto">
                    <FileBrowser {on_select} />
                </aside>
                <main class="col-span-3 p-3">
                    <CodeEditor path={(*selected).clone()} />
                </main>
            </div>
        </div>
    }
}