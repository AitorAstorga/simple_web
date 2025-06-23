// frontend_simple_web/src/pages/about.rs
use yew::prelude::*;

#[function_component(About)]
pub fn about() -> Html {
    html! {
        <section class="p-6">
            <h1 class="text-2xl font-bold mb-2">{ "About" }</h1>
            <p>{ "This admin interface is built with Yew and powered by a Rocket backend." }</p>
        </section>
    }
}