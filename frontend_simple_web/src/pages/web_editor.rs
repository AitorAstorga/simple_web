// frontend_simple_web/src/pages/web_editor.rs
use yew::prelude::*;
use crate::{components::{code_editor::CodeEditor, file_browser::FileBrowser}, config_file::get_env_var};

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
                <div>
                    <h1 class="font-bold">{ "aichan's Simple Web Editor" }</h1>
                    <p>{ "Your static site is served at: " }{ get_env_var("API_URL") } { "You can change this with the API_URL env variable." }</p>
                </div>
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