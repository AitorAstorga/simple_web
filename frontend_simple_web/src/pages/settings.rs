use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use gloo::timers::callback::Interval;

use crate::api::git::{api_git_setup, api_git_pull, api_git_test, api_get_git_status, api_commit_changes, api_push_changes, api_force_pull, api_get_auto_pull_config, api_set_auto_pull_config, GitStatus, GitRepoStatus, AutoPullConfig};
use crate::api::auth;
use crate::router::Route;
use crate::components::theme_editor::ThemeEditor;
use crate::hooks::{use_git_settings, use_async_action, input_callback};

type StatusMsg = UseStateHandle<Option<(bool, String)>>;

fn status_banner(status_message: &Option<(bool, String)>) -> Html {
    if let Some((success, ref msg)) = *status_message {
        html! {
            <div class="mb-3 p-2 rounded text-sm" style={
                if success {
                    "background-color: #dcfce7; color: #166534; border: 1px solid #bbf7d0;"
                } else {
                    "background-color: #fef2f2; color: #dc2626; border: 1px solid #fecaca;"
                }
            }>
                { msg }
            </div>
        }
    } else {
        html! {}
    }
}

fn status_placeholder(status_message: &Option<(bool, String)>, text: &str) -> Html {
    html! {
        <section class="bg-card p-4 rounded border">
            <h2 class="font-bold mb-4">{"Repository Status"}</h2>
            { status_banner(status_message) }
            <div class="flex items-center gap-2 text-gray-600">
                <span class="text-sm">{ text }</span>
            </div>
        </section>
    }
}

fn refresh_git_status(handle: &UseStateHandle<Option<GitRepoStatus>>) {
    let handle = handle.clone();
    api_get_git_status(Some(move |result: Result<GitRepoStatus, String>| {
        if let Ok(status) = result {
            handle.set(Some(status));
        }
    }));
}

#[function_component(Settings)]
pub fn settings() -> Html {
    let navigator = use_navigator().unwrap();

    // Git form fields — loaded from localStorage by the hook
    let git = use_git_settings();

    let auto_pull_enabled = use_state(|| false);
    let pull_interval = use_state(|| 30u32);
    let status_message: StatusMsg = use_state(|| None);
    let git_repo_status = use_state(|| None::<GitRepoStatus>);
    let commit_message = use_state(|| "Updated files via simple_web".to_string());
    let _status_poll_timer = use_state(|| None::<Interval>);

    // --- Async actions using the hook (replaces 6 separate is_loading states) ---

    let setup_action = {
        let git_config = git.to_config();
        use_async_action(status_message.clone(), move |cb| {
            api_git_setup(git_config.clone(), Some(cb));
        })
    };

    let test_action = {
        let git_config = git.to_config();
        use_async_action(status_message.clone(), move |cb| {
            api_git_test(git_config.clone(), Some(cb));
        })
    };

    let pull_action = use_async_action(status_message.clone(), |cb| {
        api_git_pull(Some(cb));
    });

    let push_action = use_async_action(status_message.clone(), |cb| {
        api_push_changes(Some(cb));
    });

    let commit_action = {
        let msg = commit_message.clone();
        use_async_action(status_message.clone(), move |cb| {
            api_commit_changes(msg.to_string(), Some(cb));
        })
    };

    let force_pull_action = use_async_action(status_message.clone(), |cb| {
        api_force_pull(Some(cb));
    });

    // Load auto-pull config from backend
    {
        let auto_pull_enabled = auto_pull_enabled.clone();
        let pull_interval = pull_interval.clone();

        use_effect_with((), move |_| {
            api_get_auto_pull_config(Some({
                let auto_pull_enabled = auto_pull_enabled.clone();
                let pull_interval = pull_interval.clone();
                move |result: Result<AutoPullConfig, String>| {
                    match result {
                        Ok(config) => {
                            auto_pull_enabled.set(config.enabled);
                            pull_interval.set(config.interval_minutes);
                        }
                        Err(_) => {
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
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
                        }
                    }
                }
            }));

            || ()
        });
    }

    // Poll git status regularly
    {
        let git_repo_status = git_repo_status.clone();
        let status_poll_timer = _status_poll_timer.clone();

        use_effect_with((), move |_| {
            let poll_git_status = {
                let git_repo_status = git_repo_status.clone();
                move || refresh_git_status(&git_repo_status)
            };

            let timer_for_async = status_poll_timer.clone();
            spawn_local(async move {
                gloo::timers::future::TimeoutFuture::new(2000).await;
                poll_git_status();
                let timer = Interval::new(5000, poll_git_status);
                timer_for_async.set(Some(timer));
            });

            move || { status_poll_timer.set(None); }
        });
    }

    let on_url_change = input_callback(git.repo_url.clone(), "git_repo_url");
    let on_branch_change = input_callback(git.branch.clone(), "git_branch");
    let on_username_change = input_callback(git.username.clone(), "git_username");
    let on_token_change = input_callback(git.token.clone(), "git_token");

    let on_auto_pull_toggle = {
        let auto_pull_enabled = auto_pull_enabled.clone();
        let pull_interval = pull_interval.clone();
        let status_message = status_message.clone();

        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let enabled = input.checked();

                let auto_pull_enabled = auto_pull_enabled.clone();
                let status_message = status_message.clone();
                api_set_auto_pull_config(
                    AutoPullConfig { enabled, interval_minutes: *pull_interval },
                    Some(move |result: Result<GitStatus, String>| {
                        match result {
                            Ok(status) if status.success => {
                                auto_pull_enabled.set(enabled);
                                status_message.set(Some((true, format!("Auto-pull {} successfully",
                                    if enabled { "enabled" } else { "disabled" }))));
                            }
                            Ok(status) => status_message.set(Some((false, format!("Failed to update auto-pull: {}", status.message)))),
                            Err(e) => status_message.set(Some((false, format!("Failed to update auto-pull: {}", e)))),
                        }
                    }),
                );
            }
        })
    };

    let on_interval_change = {
        let pull_interval = pull_interval.clone();
        let auto_pull_enabled = auto_pull_enabled.clone();
        let status_message = status_message.clone();

        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(minutes) = input.value().parse::<u32>() {
                    if minutes > 0 && minutes <= 1440 {
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.set_item("pull_interval", &minutes.to_string());
                            }
                        }

                        if *auto_pull_enabled {
                            let pull_interval = pull_interval.clone();
                            let status_message = status_message.clone();
                            api_set_auto_pull_config(
                                AutoPullConfig { enabled: true, interval_minutes: minutes },
                                Some(move |result: Result<GitStatus, String>| {
                                    match result {
                                        Ok(status) if status.success => {
                                            pull_interval.set(minutes);
                                            status_message.set(Some((true, format!("Auto-pull interval updated to {} minutes", minutes))));
                                        }
                                        Ok(status) => status_message.set(Some((false, format!("Failed to update interval: {}", status.message)))),
                                        Err(e) => status_message.set(Some((false, format!("Failed to update interval: {}", e)))),
                                    }
                                }),
                            );
                        } else {
                            pull_interval.set(minutes);
                        }
                    }
                }
            }
        })
    };

    let on_commit_message_change = {
        let commit_message = commit_message.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                commit_message.set(input.value());
            }
        })
    };

    // Force pull needs a confirmation dialog, so wrap the action trigger
    let force_pull_with_confirm = {
        let trigger = force_pull_action.trigger.clone();
        Callback::from(move |_| {
            if web_sys::window()
                .and_then(|w| w.confirm_with_message("WARNING: Force pull will OVERWRITE all local changes! Are you sure?").ok())
                .unwrap_or(false)
            {
                trigger.emit(());
            }
        })
    };

    // --- Render ---

    let repo_status_section = if let Some(ref status) = *git_repo_status {
        if status.success {
            html! {
                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"Repository Status"}</h2>
                    { status_banner(&status_message) }

                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <div class="flex items-center gap-2">
                                <span class="text-sm font-medium">{"Current Branch:"}</span>
                                <code class="bg-surface px-2 py-1 rounded text-sm">
                                    { status.current_branch.as_ref().unwrap_or(&"unknown".to_string()) }
                                </code>
                            </div>
                            { if status.behind_count > 0 {
                                html! {
                                    <span class="text-sm text-orange-600">{format!("{} behind", status.behind_count)}</span>
                                }
                            } else if status.ahead_count > 0 {
                                html! {
                                    <span class="text-sm text-blue-600">{format!("{} ahead", status.ahead_count)}</span>
                                }
                            } else {
                                html! {
                                    <span class="text-sm text-green-600">{"Up to date"}</span>
                                }
                            }}
                        </div>

                        { if status.has_changes {
                            html! {
                                <div class="space-y-3 mt-4">
                                    <div class="text-sm font-medium text-orange-600">{"Local Changes Detected:"}</div>
                                    { for status.changed_files.iter().map(|file| html! {
                                        <div class="flex items-center gap-2 text-xs bg-surface p-2 rounded">
                                            <span class={match file.status.as_str() {
                                                "modified" => "text-orange-500",
                                                "deleted" => "text-red-500",
                                                "staged_new" | "staged_modified" | "staged_deleted" | "staged_renamed" => "text-green-500",
                                                _ => "text-blue-500"
                                            }}>
                                                { match file.status.as_str() {
                                                    "modified" => "M",
                                                    "deleted" => "D",
                                                    "staged_new" => "A+",
                                                    "staged_modified" => "M+",
                                                    "staged_deleted" => "D+",
                                                    "staged_renamed" => "R+",
                                                    _ => "?"
                                                }}
                                            </span>
                                            <code class="text-xs">{ &file.path }</code>
                                        </div>
                                    }) }
                                    { for status.untracked_files.iter().map(|file| html! {
                                        <div class="flex items-center gap-2 text-xs bg-surface p-2 rounded">
                                            <span class="text-gray-500">{"??"}</span>
                                            <code class="text-xs">{ file }</code>
                                        </div>
                                    }) }
                                </div>
                            }
                        } else {
                            html! {
                                <div class="text-sm text-green-600 mt-4">{"Working directory clean"}</div>
                            }
                        }}

                        // Action buttons depending on state
                        { if status.has_changes {
                            html! {
                                <div class="space-y-3 pt-4 mt-4 border-t">
                                    <div>
                                        <label class="block text-sm font-medium mb-1">{"Commit Message"}</label>
                                        <input
                                            type="text"
                                            class="input w-full"
                                            placeholder="Updated files via simple_web"
                                            value={(*commit_message).clone()}
                                            onchange={on_commit_message_change}
                                        />
                                    </div>
                                    <div class="flex gap-2 flex-wrap">
                                        <button class="btn btn-primary" onclick={commit_action.trigger.reform(|_| ())} disabled={*commit_action.is_loading}>
                                            { if *commit_action.is_loading { "Committing..." } else { "Commit Changes" } }
                                        </button>
                                        <button class="btn btn-danger" onclick={force_pull_with_confirm.clone()} disabled={*force_pull_action.is_loading}>
                                            { if *force_pull_action.is_loading { "Force Pulling..." } else { "Force Pull (Discard Changes)" } }
                                        </button>
                                    </div>
                                </div>
                            }
                        } else if status.ahead_count > 0 {
                            html! {
                                <div class="pt-4 mt-4 border-t space-y-3">
                                    <span class="text-sm text-blue-600">{format!("You have {} unpushed commit(s)", status.ahead_count)}</span>
                                    <div class="flex gap-2 flex-wrap">
                                        <button class="btn btn-primary" onclick={push_action.trigger.reform(|_| ())} disabled={*push_action.is_loading}>
                                            { if *push_action.is_loading { "Pushing..." } else { "Push Changes" } }
                                        </button>
                                        <button class="btn btn-danger" onclick={force_pull_with_confirm.clone()} disabled={*force_pull_action.is_loading}>
                                            { if *force_pull_action.is_loading { "Force Pulling..." } else { "Force Pull (Discard Changes)" } }
                                        </button>
                                    </div>
                                    { if status.behind_count > 0 {
                                        html! {
                                            <div class="text-sm text-orange-600 px-2 py-1 bg-orange-50 rounded">
                                                {"Cannot pull: you are both ahead and behind. Push first or use Force Pull."}
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            }
                        } else if status.behind_count > 0 {
                            html! {
                                <div class="pt-4 mt-4 border-t">
                                    <button class="btn btn-primary" onclick={pull_action.trigger.reform(|_| ())} disabled={*pull_action.is_loading}>
                                        { if *pull_action.is_loading { "Pulling..." } else { "Pull Updates" } }
                                    </button>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </section>
            }
        } else {
            status_placeholder(&status_message, "No Git repository configured. Please setup a repository below to enable version control.")
        }
    } else {
        status_placeholder(&status_message, "Checking repository status...")
    };

    html! {
        <div class="settings-page p-3">
            <header class="mb-4">
                <div class="flex items-center justify-between mb-4 w-full">
                    <div>
                        <h1 class="font-bold text-xl mb-2">{"Settings"}</h1>
                        <p class="text-sm">{"Configure git repository and automatic synchronization"}</p>
                    </div>
                    <div class="flex items-center gap-2">
                        <button
                            class="btn btn-danger text-sm"
                            onclick={{
                                let navigator = navigator.clone();
                                Callback::from(move |_| {
                                    auth::logout();
                                    navigator.push(&Route::Login);
                                })
                            }}
                        >
                            { "Logout" }
                        </button>
                        <Link<Route> to={Route::WebEditor} classes="btn btn-secondary text-sm">
                            { "Back to Editor" }
                        </Link<Route>>
                    </div>
                </div>
            </header>

            <div class="space-y-6">
                { repo_status_section }

                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"Git Repository Configuration"}</h2>
                    <div>
                        <div>
                            <label class="block text-sm font-medium mb-1">{"Repository URL *"}</label>
                            <input type="url" class="input w-full" placeholder="https://github.com/username/repo.git"
                                value={(*git.repo_url).clone()} onchange={on_url_change} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium mb-1">{"Branch (optional)"}</label>
                            <input type="text" class="input w-full" placeholder="main"
                                value={(*git.branch).clone()} onchange={on_branch_change} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium mb-1">{"Username (for private repos)"}</label>
                            <input type="text" class="input w-full" placeholder="your-username"
                                value={(*git.username).clone()} onchange={on_username_change} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium mb-1">{"Personal Access Token (for private repos)"}</label>
                            <input type="password" class="input w-full" placeholder="ghp_..."
                                value={(*git.token).clone()} onchange={on_token_change} />
                        </div>
                        <div class="flex gap-2">
                            <button class="btn btn-secondary" onclick={test_action.trigger.reform(|_| ())} disabled={*test_action.is_loading}>
                                { if *test_action.is_loading { "Testing..." } else { "Test Connection" } }
                            </button>
                            <button class="btn btn-primary" onclick={setup_action.trigger.reform(|_| ())} disabled={*setup_action.is_loading}>
                                { if *setup_action.is_loading { "Setting up..." } else { "Setup & Clone Repository" } }
                            </button>
                            <button class="btn btn-secondary" onclick={pull_action.trigger.reform(|_| ())} disabled={*pull_action.is_loading}>
                                { if *pull_action.is_loading { "Pulling..." } else { "Pull Updates" } }
                            </button>
                        </div>
                    </div>
                </section>

                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"Automatic Synchronization"}</h2>
                    <div>
                        <div class="flex items-center gap-3">
                            <input type="checkbox" id="auto-pull" checked={*auto_pull_enabled} onchange={on_auto_pull_toggle} />
                            <label for="auto-pull" class="text-sm font-medium">{"Enable automatic pulling"}</label>
                        </div>
                        { if *auto_pull_enabled {
                            html! {
                                <div>
                                    <label class="block text-sm font-medium mb-1">{"Pull interval (minutes)"}</label>
                                    <input type="number" min="1" max="1440" class="input w-32"
                                        value={(*pull_interval).to_string()} onchange={on_interval_change} />
                                    <p class="text-sm mt-1">{format!("Automatically check for changes and pull every {} minutes. Repository must be set up first.", *pull_interval)}</p>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </section>

                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"Editor Theme Configuration"}</h2>
                    <ThemeEditor />
                </section>

                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"Actions"}</h2>
                    <div class="space-y-2">
                        <button
                            class="btn btn-danger text-sm"
                            onclick={Callback::from(|_| {
                                if let Some(window) = web_sys::window() {
                                    if let Ok(Some(storage)) = window.local_storage() {
                                        for key in ["git_repo_url", "git_branch", "git_username", "git_token", "auto_pull_enabled", "pull_interval"] {
                                            let _ = storage.remove_item(key);
                                        }
                                        web_sys::window().unwrap().location().reload().unwrap();
                                    }
                                }
                            })}
                        >
                            {"Clear All Settings"}
                        </button>
                        <p class="text-sm">{"This will clear all saved git configuration and reload the page"}</p>
                    </div>
                </section>
            </div>
        </div>
    }
}
