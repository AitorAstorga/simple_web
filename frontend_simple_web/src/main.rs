mod api;
mod components;
mod pages;
mod router;

use crate::router::AppRouter;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
    <BrowserRouter>
        <AppRouter />
    </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}