// frontend_simple_web/src/hooks/use_async_action.rs
//
// Encapsulates the repeated pattern:
//   1. Set loading=true, clear status
//   2. Call API with callback
//   3. On response: set loading=false, set status from result

use yew::prelude::*;
use crate::api::git::GitStatus;

type StatusMsg = UseStateHandle<Option<(bool, String)>>;

pub struct AsyncAction {
    pub is_loading: UseStateHandle<bool>,
    pub trigger: Callback<()>,
}

/// Creates a reusable loading + status callback pattern.
///
/// `action_fn` receives a boxed callback that it should pass to the API function.
/// The callback automatically clears loading and sets the status message.
///
/// `status_message` is a shared handle: all actions on the same page can share it.
#[hook]
pub fn use_async_action<F>(status_message: StatusMsg, action_fn: F) -> AsyncAction
where
    F: Fn(Box<dyn Fn(Result<GitStatus, String>)>) + 'static,
{
    let is_loading = use_state(|| false);

    let trigger = {
        let is_loading = is_loading.clone();
        let status_message = status_message.clone();
        Callback::from(move |_: ()| {
            is_loading.set(true);
            status_message.set(None);
            let loading = is_loading.clone();
            let status_msg = status_message.clone();
            action_fn(Box::new(move |result: Result<GitStatus, String>| {
                loading.set(false);
                match result {
                    Ok(s) => status_msg.set(Some((s.success, s.message))),
                    Err(e) => status_msg.set(Some((false, e))),
                }
            }));
        })
    };

    AsyncAction { is_loading, trigger }
}
