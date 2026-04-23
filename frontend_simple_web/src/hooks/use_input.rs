// frontend_simple_web/src/hooks/use_input.rs
//
// Shared input-change callback that persists the value to localStorage.

use yew::prelude::*;
use web_sys::HtmlInputElement;

fn save_setting(key: &str, value: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}

/// Creates a Callback<Event> that reads the input value, writes it to
/// `localStorage[storage_key]`, and pushes it into `state`.
pub fn input_callback(state: UseStateHandle<String>, storage_key: &'static str) -> Callback<Event> {
    Callback::from(move |e: Event| {
        if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
            let value = input.value();
            save_setting(storage_key, &value);
            state.set(value);
        }
    })
}
