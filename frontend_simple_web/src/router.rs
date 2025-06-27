// frontend_simple_web/src/router.rs
use crate::pages::login::Login;
use crate::pages::web_editor::WebEditor;
use crate::pages::about::About;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Login,
    #[at("/editor")]
    WebEditor,
    #[at("/about")]
    About,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(AppRouter)]
pub fn app_router() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Login => html! { <Login /> },
        Route::WebEditor => html! { <WebEditor /> },
        Route::About => html! { <About /> },
        Route::NotFound => html! { <h1>{ "404 - Page not found" }</h1> },
    }
}