// frontend_simple_web/src/components/git_manager.rs
use yew::prelude::*;
use web_sys::HtmlInputElement;

use crate::api::git::{api_git_setup, api_git_pull, GitRepoConfig, GitStatus};

#[function_component(GitManager)]
pub fn git_manager() -> Html {
    let show_form = use_state(|| false);
    let repo_url = use_state(|| String::new());
    let branch = use_state(|| String::new());
    let username = use_state(|| String::new());
    let token = use_state(|| String::new());
    let status_message = use_state(|| None::<String>);
    let is_loading = use_state(|| false);

    let toggle_form = {
        let show_form = show_form.clone();
        Callback::from(move |_| {
            show_form.set(!*show_form);
        })
    };

    let on_url_change = {
        let repo_url = repo_url.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                repo_url.set(input.value());
            }
        })
    };

    let on_branch_change = {
        let branch = branch.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                branch.set(input.value());
            }
        })
    };

    let on_username_change = {
        let username = username.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                username.set(input.value());
            }
        })
    };

    let on_token_change = {
        let token = token.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                token.set(input.value());
            }
        })
    };

    let setup_repo = {
        let repo_url = repo_url.clone();
        let branch = branch.clone();
        let username = username.clone();
        let token = token.clone();
        let status_message = status_message.clone();
        let is_loading = is_loading.clone();
        let show_form = show_form.clone();
        
        Callback::from(move |_| {
            if repo_url.trim().is_empty() {
                status_message.set(Some("Repository URL is required".to_string()));
                return;
            }
            
            is_loading.set(true);
            status_message.set(None);
            
            let config = GitRepoConfig {
                url: repo_url.trim().to_string(),
                branch: if branch.trim().is_empty() { None } else { Some(branch.trim().to_string()) },
                username: if username.trim().is_empty() { None } else { Some(username.trim().to_string()) },
                token: if token.trim().is_empty() { None } else { Some(token.trim().to_string()) },
            };
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading.clone();
            let show_form_clone = show_form.clone();
            
            api_git_setup(config, Some(move |result: Result<GitStatus, String>| {
                is_loading_clone.set(false);
                match result {
                    Ok(status) => {
                        if status.success {
                            status_msg_clone.set(Some(format!("✅ {}", status.message)));
                            show_form_clone.set(false);
                        } else {
                            status_msg_clone.set(Some(format!("❌ {}", status.message)));
                        }
                    }
                    Err(e) => {
                        status_msg_clone.set(Some(format!("❌ {}", e)));
                    }
                }
            }));
        })
    };

    let pull_repo = {
        let status_message = status_message.clone();
        let is_loading = is_loading.clone();
        
        Callback::from(move |_| {
            is_loading.set(true);
            status_message.set(None);
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading.clone();
            
            api_git_pull(Some(move |result: Result<GitStatus, String>| {
                is_loading_clone.set(false);
                match result {
                    Ok(status) => {
                        if status.success {
                            status_msg_clone.set(Some(format!("✅ {}", status.message)));
                            // Reload the page to show updated files
                            let _ = web_sys::window().map(|w| w.location().reload());
                        } else {
                            status_msg_clone.set(Some(format!("❌ {}", status.message)));
                        }
                    }
                    Err(e) => {
                        status_msg_clone.set(Some(format!("❌ {}", e)));
                    }
                }
            }));
        })
    };

    html! {
        <div class="git-manager border-t pt-4 mt-4">
            <div class="flex items-center gap-2 mb-2">
                <h3 class="font-bold">{"🔧 Git Repository"}</h3>
                <button 
                    class="btn btn-primary text-sm"
                    onclick={toggle_form}
                    disabled={*is_loading}
                >
                    { if *show_form { "Cancel" } else { "Setup" } }
                </button>
                <button 
                    class="btn btn-secondary text-sm"
                    onclick={pull_repo}
                    disabled={*is_loading}
                >
                    { if *is_loading { "⏳ Pulling..." } else { "🔄 Pull" } }
                </button>
            </div>
            
            { if let Some(ref msg) = *status_message {
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
                                value={(*repo_url).clone()}
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
                                value={(*branch).clone()}
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
                                value={(*username).clone()}
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
                                value={(*token).clone()}
                                onchange={on_token_change}
                            />
                        </div>
                        
                        <button 
                            class="btn btn-primary w-full"
                            onclick={setup_repo}
                            disabled={*is_loading}
                        >
                            { if *is_loading { "⏳ Setting up..." } else { "Setup Repository" } }
                        </button>
                    </div>
                }
            } else {
                html! {}
            }}
        </div>
    }
}