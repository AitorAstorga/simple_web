// frontend_simple_web/src/components/auth_guard.rs
use yew::prelude::*;
use yew_router::prelude::*;
use crate::{api::auth, router::Route};

#[derive(Properties, PartialEq)]
pub struct AuthGuardProps {
    pub children: Children,
}

#[function_component(AuthGuard)]
pub fn auth_guard(props: &AuthGuardProps) -> Html {
    let _navigator = use_navigator().unwrap();
    let is_authenticated = use_state(|| false);
    
    // Check authentication status on component mount
    {
        let is_authenticated = is_authenticated.clone();
        use_effect_with((), move |_| {
            // Check authentication immediately and update state
            let authenticated = auth::is_authenticated();
            is_authenticated.set(authenticated);
            || ()
        });
    }

    if *is_authenticated {
        html! {
            <div class="authenticated-content">
                { for props.children.iter() }
            </div>
        }
    } else {
        html! {
            <div class="auth-notice">
                <div class="auth-notice-card">
                    <h2>{"🔒 Authentication Required"}</h2>
                    <p>{"You need to be logged in to access this page."}</p>
                    <Link<Route> to={Route::Login} classes="btn btn-primary">
                        {"Go to Login"}
                    </Link<Route>>
                </div>
            </div>
        }
    }
}