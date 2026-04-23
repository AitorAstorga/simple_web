// frontend_simple_web/src/components/git_manager.rs
use yew::prelude::*;

use crate::api::git::{api_git_setup, api_git_pull, GitStatus};
use crate::hooks::{use_git_settings, use_async_action, input_callback};

#[function_component(GitManager)]
pub fn git_manager() -> Html {
    let show_form = use_state(|| false);
    let status_message: UseStateHandle<Option<(bool, String)>> = use_state(|| None);

    let git = use_git_settings();

    let setup_action = {
        let git_config = git.to_config();
        let show_form = show_form.clone();
        use_async_action(status_message.clone(), move |cb| {
            let show_form = show_form.clone();
            let config = git_config.clone();
            let inner_cb = move |result: Result<GitStatus, String>| {
                if let Ok(ref s) = result {
                    if s.success {
                        show_form.set(false);
                    }
                }
                cb(result);
            };
            api_git_setup(config, Some(inner_cb));
        })
    };

    let pull_action = use_async_action(status_message.clone(), |cb| {
        api_git_pull(Some(move |result: Result<GitStatus, String>| {
            if let Ok(ref s) = result {
                if s.success {
                    let _ = web_sys::window().map(|w| w.location().reload());
                }
            }
            cb(result);
        }));
    });

    let toggle_form = {
        let show_form = show_form.clone();
        Callback::from(move |_| {
            show_form.set(!*show_form);
        })
    };

    let on_url_change = input_callback(git.repo_url.clone(), "git_repo_url");
    let on_branch_change = input_callback(git.branch.clone(), "git_branch");
    let on_username_change = input_callback(git.username.clone(), "git_username");
    let on_token_change = input_callback(git.token.clone(), "git_token");

    let is_loading = *setup_action.is_loading || *pull_action.is_loading;

    html! {
        <div class="git-manager border-t pt-4 mt-4">
            <div class="flex items-center gap-2 mb-2">
                <h3 class="font-bold">{"Git Repository"}</h3>
                <button
                    class="btn btn-primary text-sm"
                    onclick={toggle_form}
                    disabled={is_loading}
                >
                    { if *show_form { "Cancel" } else { "Setup" } }
                </button>
                <button
                    class="btn btn-secondary text-sm"
                    onclick={pull_action.trigger.reform(|_| ())}
                    disabled={is_loading}
                >
                    { if *pull_action.is_loading { "Pulling..." } else { "Pull" } }
                </button>
            </div>

            { if let Some((_, ref msg)) = *status_message {
                html! {
                    <div class="mb-2 p-2 rounded text-sm bg-card">
                        { msg }
                    </div>
                }
            } else {
                html! {}
            }}

            { if *show_form {
                html! {
                    <div class="space-y-2 p-3 border rounded bg-card">
                        <div>
                            <label class="block text-sm font-medium mb-1">
                                {"Repository URL *"}
                            </label>
                            <input
                                type="url"
                                class="input w-full text-sm"
                                placeholder="https://github.com/username/repo.git"
                                value={(*git.repo_url).clone()}
                                onchange={on_url_change}
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium mb-1">
                                {"Branch (optional)"}
                            </label>
                            <input
                                type="text"
                                class="input w-full text-sm"
                                placeholder="main"
                                value={(*git.branch).clone()}
                                onchange={on_branch_change}
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium mb-1">
                                {"Username (for private repos)"}
                            </label>
                            <input
                                type="text"
                                class="input w-full text-sm"
                                placeholder="your-username"
                                value={(*git.username).clone()}
                                onchange={on_username_change}
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium mb-1">
                                {"Personal Access Token (for private repos)"}
                            </label>
                            <input
                                type="password"
                                class="input w-full text-sm"
                                placeholder="ghp_..."
                                value={(*git.token).clone()}
                                onchange={on_token_change}
                            />
                        </div>

                        <button
                            class="btn btn-primary w-full"
                            onclick={setup_action.trigger.reform(|_| ())}
                            disabled={*setup_action.is_loading}
                        >
                            { if *setup_action.is_loading { "Setting up..." } else { "Setup Repository" } }
                        </button>
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}
