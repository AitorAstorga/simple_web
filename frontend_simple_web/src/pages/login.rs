use yew::prelude::*;
use web_sys::HtmlInputElement;
use yew_router::prelude::*;
use crate::{api::auth, config_file::load_config, router::Route};
use wasm_bindgen_futures::spawn_local;

#[function_component(Login)]
pub fn login() -> Html {
    let input_password = use_state(|| String::new());
    let ready = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let is_loading = use_state(|| false);
    let navigator = use_navigator().unwrap();

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
        let password = input_password.clone();
        Callback::from(move |e: InputEvent| {
            let value = e
                .target_unchecked_into::<HtmlInputElement>()
                .value();
            password.set(value);
        })
    };

    let onclick = {
        let password = (*input_password).clone();
        let error_message = error_message.clone();
        let is_loading = is_loading.clone();
        let navigator = navigator.clone();
        
        Callback::from(move |_| {
            let password = password.clone();
            let error_message = error_message.clone();
            let is_loading = is_loading.clone();
            let navigator = navigator.clone();
            
            spawn_local(async move {
                is_loading.set(true);
                error_message.set(None);
                
                match auth::login(&password).await {
                    Ok(_token) => {
                        navigator.push(&Route::WebEditor);
                    }
                    Err(err) => {
                        error_message.set(Some(err));
                    }
                }
                
                is_loading.set(false);
            });
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
                
                { if let Some(error) = (*error_message).as_ref() {
                    html! { <div class="error-message mb-2" style="color: red;">{ error }</div> }
                } else {
                    html! {}
                }}
                
                <input
                    class="input"
                    type="password"
                    placeholder="Password"
                    value={(*input_password).clone()}
                    {oninput}
                    disabled={*is_loading}
                />

                <button 
                    class="btn btn-primary enter_password" 
                    onclick={onclick}
                    disabled={*is_loading || input_password.is_empty()}
                >
                    { if *is_loading { "Logging in..." } else { "Enter" } }
                </button>
            </section>
        </div>
    }
}