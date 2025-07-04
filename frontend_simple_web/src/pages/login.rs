// src/components/Login.rs
use yew::prelude::*;
use web_sys::HtmlInputElement;
use yew_router::prelude::*;
use crate::{api::auth, config_file::load_config, router::Route};
use wasm_bindgen_futures::spawn_local;

#[function_component(Login)]
pub fn login() -> Html {
    // local state for the input field
    let input_token = use_state(|| String::new());
    let ready = use_state(|| false);

    {
        let ready = ready.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                load_config().await;
                ready.set(true);
            });
            || ()
        });
    }

    let oninput = {
        let token = input_token.clone();
        Callback::from(move |e: InputEvent| {
            let value = e
                .target_unchecked_into::<HtmlInputElement>()
                .value();
            token.set(value);
        })
    };

    let onclick = {
        let token = (*input_token).clone();
        Callback::from(move |_| {
            auth::set_token(&token);
        })
    };

    if !*ready {
        return html! { "Loading..." };
    }

    html! {
        <div class="login-container flex justify-center">
            <section class="login-card">
                <div class="flex justify-center">
                    <img src="static/img/aichan.svg" alt="aichan" class="login-logo" />
                </div>
                <h1 class="mb-2">{ "Login" }</h1>
                <input
                    class="input"
                    type="password"
                    placeholder="Password"
                    value={(*input_token).clone()}
                    {oninput}
                />

                <Link<Route> to={Route::WebEditor} classes="btn btn-primary">
                    <button class="enter_password" {onclick}>{ "Enter" }</button>
                </Link<Route>>
            </section>
        </div>
    }
}