// frontend_simple_web/src/hooks/mod.rs

mod use_git_settings;
mod use_async_action;
mod use_input;

pub use use_git_settings::use_git_settings;
pub use use_async_action::use_async_action;
pub use use_input::input_callback;
