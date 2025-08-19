use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use gloo::timers::callback::Interval;

use crate::api::git::{api_git_setup, api_git_pull, api_git_test, api_get_auto_pull_config, api_set_auto_pull_config, GitRepoConfig, GitStatus, AutoPullConfig};
use crate::router::Route;

#[function_component(Settings)]
pub fn settings() -> Html {
    let repo_url = use_state(|| String::new());
    let branch = use_state(|| String::new());
    let username = use_state(|| String::new());
    let token = use_state(|| String::new());
    let auto_pull_enabled = use_state(|| false);
    let pull_interval = use_state(|| 30); // minutes
    let status_message = use_state(|| None::<String>);
    let is_loading_test = use_state(|| false);
    let is_loading_setup = use_state(|| false);
    let is_loading_pull = use_state(|| false);
    let _auto_pull_timer = use_state(|| None::<Interval>);

    // Load settings from localStorage
    {
        let repo_url = repo_url.clone();
        let branch = branch.clone();
        let username = username.clone();
        let token = token.clone();
        let auto_pull_enabled = auto_pull_enabled.clone();
        let pull_interval = pull_interval.clone();
        
        use_effect_with((), move |_| {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(url)) = storage.get_item("git_repo_url") {
                        repo_url.set(url);
                    }
                    if let Ok(Some(br)) = storage.get_item("git_branch") {
                        branch.set(br);
                    }
                    if let Ok(Some(user)) = storage.get_item("git_username") {
                        username.set(user);
                    }
                    if let Ok(Some(tok)) = storage.get_item("git_token") {
                        token.set(tok);
                    }
                    if let Ok(Some(auto)) = storage.get_item("auto_pull_enabled") {
                        auto_pull_enabled.set(auto == "true");
                    }
                    if let Ok(Some(interval)) = storage.get_item("pull_interval") {
                        if let Ok(minutes) = interval.parse::<u32>() {
                            pull_interval.set(minutes);
                        }
                    }
                }
            }
            || ()
        });
    }

    let save_setting = |key: &str, value: &str| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(key, value);
            }
        }
    };

    let on_url_change = {
        let repo_url = repo_url.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let value = input.value();
                save_setting("git_repo_url", &value);
                repo_url.set(value);
            }
        })
    };

    let on_branch_change = {
        let branch = branch.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let value = input.value();
                save_setting("git_branch", &value);
                branch.set(value);
            }
        })
    };

    let on_username_change = {
        let username = username.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let value = input.value();
                save_setting("git_username", &value);
                username.set(value);
            }
        })
    };

    let on_token_change = {
        let token = token.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let value = input.value();
                save_setting("git_token", &value);
                token.set(value);
            }
        })
    };

    let on_auto_pull_toggle = {
        let auto_pull_enabled = auto_pull_enabled.clone();
        let auto_pull_timer = _auto_pull_timer.clone();
        let pull_interval = pull_interval.clone();
        
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let enabled = input.checked();
                save_setting("auto_pull_enabled", if enabled { "true" } else { "false" });
                auto_pull_enabled.set(enabled);
                
                // Setup or clear the timer
                if enabled {
                    let interval_ms = *pull_interval * 60 * 1000; // convert minutes to milliseconds
                    let timer = Interval::new(interval_ms, move || {
                        spawn_local(async {
                            // For automatic pulls, we silently try to pull and ignore failures
                            // since the user will see status updates in the settings page when they check
                            api_git_pull(Some(|_result: Result<GitStatus, String>| {
                                // Log the result but don't show UI feedback for automatic pulls
                                // Users can check the last pull status in settings if needed
                            }));
                        });
                    });
                    auto_pull_timer.set(Some(timer));
                } else {
                    auto_pull_timer.set(None);
                }
            }
        })
    };

    let on_interval_change = {
        let pull_interval = pull_interval.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(minutes) = input.value().parse::<u32>() {
                    if minutes > 0 && minutes <= 1440 { // max 24 hours
                        save_setting("pull_interval", &minutes.to_string());
                        pull_interval.set(minutes);
                    }
                }
            }
        })
    };

    let setup_repo = {
        let repo_url = repo_url.clone();
        let branch = branch.clone();
        let username = username.clone();
        let token = token.clone();
        let status_message = status_message.clone();
        let is_loading_setup = is_loading_setup.clone();
        
        Callback::from(move |_| {
            if repo_url.trim().is_empty() {
                status_message.set(Some("Repository URL is required".to_string()));
                return;
            }
            
            is_loading_setup.set(true);
            status_message.set(None);
            
            let config = GitRepoConfig {
                url: repo_url.trim().to_string(),
                branch: if branch.trim().is_empty() { None } else { Some(branch.trim().to_string()) },
                username: if username.trim().is_empty() { None } else { Some(username.trim().to_string()) },
                token: if token.trim().is_empty() { None } else { Some(token.trim().to_string()) },
            };
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading_setup.clone();
            
            api_git_setup(config, Some(move |result: Result<GitStatus, String>| {
                is_loading_clone.set(false);
                match result {
                    Ok(status) => {
                        if status.success {
                            status_msg_clone.set(Some(format!("✅ {}", status.message)));
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

    let test_connection = {
        let repo_url = repo_url.clone();
        let branch = branch.clone();
        let username = username.clone();
        let token = token.clone();
        let status_message = status_message.clone();
        let is_loading_test = is_loading_test.clone();
        
        Callback::from(move |_| {
            if repo_url.trim().is_empty() {
                status_message.set(Some("❌ Repository URL is required".to_string()));
                return;
            }
            
            is_loading_test.set(true);
            status_message.set(None);
            
            let config = GitRepoConfig {
                url: repo_url.trim().to_string(),
                branch: if branch.trim().is_empty() { None } else { Some(branch.trim().to_string()) },
                username: if username.trim().is_empty() { None } else { Some(username.trim().to_string()) },
                token: if token.trim().is_empty() { None } else { Some(token.trim().to_string()) },
            };
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading_test.clone();
            
            api_git_test(config, Some(move |result: Result<GitStatus, String>| {
                is_loading_clone.set(false);
                match result {
                    Ok(status) => {
                        if status.success {
                            status_msg_clone.set(Some(format!("✅ {}", status.message)));
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

    let pull_updates = {
        let status_message = status_message.clone();
        let is_loading_pull = is_loading_pull.clone();
        
        Callback::from(move |_| {
            is_loading_pull.set(true);
            status_message.set(None);
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading_pull.clone();
            
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
                        if e.contains("No Git repository found") {
                            status_msg_clone.set(Some("❌ No repository found. Please setup repository first.".to_string()));
                        } else {
                            status_msg_clone.set(Some(format!("❌ {}", e)));
                        }
                    }
                }
            }));
        })
    };

    html! {
        <div class="settings-page p-3">
            <header class="mb-4">
                <div class="flex items-start justify-between mb-4">
                    <div>
                        <h1 class="font-bold text-xl mb-2">{"⚙️ Settings"}</h1>
                        <p class="text-sm">{"Configure git repository and automatic synchronization"}</p>
                    </div>
                    <Link<Route> to={Route::WebEditor} classes="btn btn-secondary text-sm">
                        { "← Back to Editor" }
                    </Link<Route>>
                </div>
            </header>

            <div class="space-y-6">
                // Git Repository Configuration
                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"🔧 Git Repository Configuration"}</h2>
                    
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium mb-1">
                                {"Repository URL *"}
                            </label>
                            <input 
                                type="url"
                                class="input w-full"
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
                                class="input w-full"
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
                                class="input w-full"
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
                                class="input w-full"
                                placeholder="ghp_..."
                                value={(*token).clone()}
                                onchange={on_token_change}
                            />
                        </div>
                        
                        <div class="flex gap-2">
                            <button 
                                class="btn btn-secondary"
                                onclick={test_connection}
                                disabled={*is_loading_test}
                            >
                                { if *is_loading_test { "⏳ Testing..." } else { "🧪 Test Connection" } }
                            </button>
                            <button 
                                class="btn btn-primary"
                                onclick={setup_repo}
                                disabled={*is_loading_setup}
                            >
                                { if *is_loading_setup { "⏳ Setting up..." } else { "📁 Setup & Clone Repository" } }
                            </button>
                            <button 
                                class="btn btn-secondary"
                                onclick={pull_updates}
                                disabled={*is_loading_pull}
                            >
                                { if *is_loading_pull { "⏳ Pulling..." } else { "🔄 Pull Updates" } }
                            </button>
                        </div>
                    </div>
                </section>

                // Automatic Synchronization
                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"🔄 Automatic Synchronization"}</h2>
                    
                    <div class="space-y-4">
                        <div class="flex items-center gap-3">
                            <input 
                                type="checkbox"
                                id="auto-pull"
                                checked={*auto_pull_enabled}
                                onchange={on_auto_pull_toggle}
                            />
                            <label for="auto-pull" class="text-sm font-medium">
                                {"Enable automatic pulling"}
                            </label>
                        </div>
                        
                        { if *auto_pull_enabled {
                            html! {
                                <div>
                                    <label class="block text-sm font-medium mb-1">
                                        {"Pull interval (minutes)"}
                                    </label>
                                    <input 
                                        type="number"
                                        min="1"
                                        max="1440"
                                        class="input w-32"
                                        value={(*pull_interval).to_string()}
                                        onchange={on_interval_change}
                                    />
                                    <p class="text-sm mt-1">{format!("Automatically check for changes and pull every {} minutes. Repository must be set up first.", *pull_interval)}</p>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </section>

                // Status Messages
                { if let Some(ref msg) = *status_message {
                    html! {
                        <div class="bg-card p-3 rounded border">
                            <h3 class="font-medium mb-2">{"Status"}</h3>
                            <div class="text-sm">
                                { msg }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}

                // Actions
                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"🧹 Actions"}</h2>
                    
                    <div class="space-y-2">
                        <button 
                            class="btn btn-danger text-sm"
                            onclick={Callback::from(|_| {
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        let _ = storage.remove_item("git_repo_url");
                                        let _ = storage.remove_item("git_branch");
                                        let _ = storage.remove_item("git_username");
                                        let _ = storage.remove_item("git_token");
                                        let _ = storage.remove_item("auto_pull_enabled");
                                        let _ = storage.remove_item("pull_interval");
                                        web_sys::window().unwrap().location().reload().unwrap();
                                    }
                                }
                            })}
                        >
                            {"🗑️ Clear All Settings"}
                        </button>
                        <p class="text-sm">{"This will clear all saved git configuration and reload the page"}</p>
                    </div>
                </section>
            </div>
        </div>
    }
}