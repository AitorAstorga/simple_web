// frontend_simple_web/src/pages/web_editor.rs
use yew::prelude::*;
use yew_router::prelude::*;
use crate::{components::{code_editor::CodeEditor, file_browser::FileBrowser}, config_file::get_env_var, router::Route, api::auth};

#[function_component(WebEditor)]
pub fn web_editor() -> Html {
    let selected = use_state(|| None as Option<String>);
    let navigator = use_navigator().unwrap();

    let on_select = {
        let selected = selected.clone();
        Callback::from(move |p: String| selected.set(Some(p)))
    };

    let logout_callback = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            auth::logout();
            navigator.push(&Route::Login);
        })
    };

    html! {
        <div>
            <header class="flex items-center justify-between">
                <div class="flex items-center">
                    <img src="static/img/aichan.svg" alt="aichan" class="logo" />
                    <div>
                        <h1 class="font-bold">{ "aichan's Simple Web Editor" }</h1>
                        <p>{ "Your static site is served at: " } <a href={ get_env_var("API_URL") } target="_blank">{ get_env_var("API_URL") }</a> { " You can change this with the "} <code>{ "API_URL" }</code> {" env variable." }</p>
                    </div>
                </div>
                <div class="flex gap-2">
                    <button class="btn btn-danger text-sm" onclick={logout_callback}>
                        { "🚪 Logout" }
                    </button>
                    <Link<Route> to={Route::Settings} classes="btn btn-secondary text-sm">
                        { "⚙️ Settings" }
                    </Link<Route>>
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