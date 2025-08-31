use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use gloo::timers::callback::Interval;

use crate::api::git::{api_git_setup, api_git_pull, api_git_test, api_get_git_status, api_commit_changes, api_push_changes, api_force_pull, api_get_auto_pull_config, api_set_auto_pull_config, GitRepoConfig, GitStatus, GitRepoStatus, AutoPullConfig};
use crate::api::auth;
use crate::router::Route;
use crate::components::theme_editor::ThemeEditor;

#[function_component(Settings)]
pub fn settings() -> Html {
    let navigator = use_navigator().unwrap();
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
    let is_loading_commit = use_state(|| false);
    let is_loading_push = use_state(|| false);
    let is_loading_force_pull = use_state(|| false);
    let git_repo_status = use_state(|| None::<GitRepoStatus>);
    let commit_message = use_state(|| "Updated files via simple_web".to_string());
    let _auto_pull_timer = use_state(|| None::<Interval>);
    let _status_poll_timer = use_state(|| None::<Interval>);

    // Load settings from localStorage and backend
    {
        let repo_url = repo_url.clone();
        let branch = branch.clone();
        let username = username.clone();
        let token = token.clone();
        let auto_pull_enabled = auto_pull_enabled.clone();
        let pull_interval = pull_interval.clone();
        
        use_effect_with((), move |_| {
            // Load local storage settings
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
                }
            }
            
            // Load auto-pull config from backend
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
                            // Fallback to localStorage if backend fails
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
                move || {
                    api_get_git_status(Some({
                        let git_repo_status = git_repo_status.clone();
                        move |result: Result<GitRepoStatus, String>| {
                            match result {
                                Ok(status) => {
                                    git_repo_status.set(Some(status));
                                }
                                Err(_) => {
                                    // Silently ignore errors for background polling
                                }
                            }
                        }
                    }));
                }
            };
            
            // Don't poll immediately to avoid showing status too quickly after setup  
            // Wait 2 seconds before first poll to let operations complete
            let timer_for_async = status_poll_timer.clone();
            spawn_local(async move {
                gloo::timers::future::TimeoutFuture::new(2000).await;
                poll_git_status();
                
                // Poll every 5 seconds after the initial delay
                let timer = Interval::new(5000, poll_git_status);
                timer_for_async.set(Some(timer));
            });
            
            move || {
                status_poll_timer.set(None);
            }
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
        let pull_interval = pull_interval.clone();
        let status_message = status_message.clone();
        
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let enabled = input.checked();
                let config = AutoPullConfig {
                    enabled,
                    interval_minutes: *pull_interval,
                };
                
                // Save to localStorage as fallback
                save_setting("auto_pull_enabled", if enabled { "true" } else { "false" });
                
                // Update backend configuration
                api_set_auto_pull_config(config, Some({
                    let auto_pull_enabled = auto_pull_enabled.clone();
                    let status_message = status_message.clone();
                    move |result: Result<GitStatus, String>| {
                        match result {
                            Ok(status) => {
                                if status.success {
                                    auto_pull_enabled.set(enabled);
                                    status_message.set(Some(format!("✅ Auto-pull {} successfully", 
                                        if enabled { "enabled" } else { "disabled" })));
                                } else {
                                    status_message.set(Some(format!("❌ Failed to update auto-pull: {}", status.message)));
                                }
                            }
                            Err(e) => {
                                status_message.set(Some(format!("❌ Failed to update auto-pull: {}", e)));
                            }
                        }
                    }
                }));
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
                    if minutes > 0 && minutes <= 1440 { // max 24 hours
                        let config = AutoPullConfig {
                            enabled: *auto_pull_enabled,
                            interval_minutes: minutes,
                        };
                        
                        // Save to localStorage as fallback
                        save_setting("pull_interval", &minutes.to_string());
                        
                        // Update backend configuration if auto-pull is enabled
                        if *auto_pull_enabled {
                            api_set_auto_pull_config(config, Some({
                                let pull_interval = pull_interval.clone();
                                let status_message = status_message.clone();
                                move |result: Result<GitStatus, String>| {
                                    match result {
                                        Ok(status) => {
                                            if status.success {
                                                pull_interval.set(minutes);
                                                status_message.set(Some(format!("✅ Auto-pull interval updated to {} minutes", minutes)));
                                            } else {
                                                status_message.set(Some(format!("❌ Failed to update interval: {}", status.message)));
                                            }
                                        }
                                        Err(e) => {
                                            status_message.set(Some(format!("❌ Failed to update interval: {}", e)));
                                        }
                                    }
                                }
                            }));
                        } else {
                            pull_interval.set(minutes);
                        }
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

    let on_commit_message_change = {
        let commit_message = commit_message.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                commit_message.set(input.value());
            }
        })
    };

    let commit_changes = {
        let commit_message = commit_message.clone();
        let status_message = status_message.clone();
        let is_loading_commit = is_loading_commit.clone();
        let git_repo_status = git_repo_status.clone();
        
        Callback::from(move |_| {
            if commit_message.trim().is_empty() {
                status_message.set(Some("❌ Commit message is required".to_string()));
                return;
            }
            
            is_loading_commit.set(true);
            status_message.set(None);
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading_commit.clone();
            let git_status_clone = git_repo_status.clone();
            let message = commit_message.to_string();
            
            api_commit_changes(message, Some(move |result: Result<GitStatus, String>| {
                is_loading_clone.set(false);
                match result {
                    Ok(status) => {
                        if status.success {
                            status_msg_clone.set(Some(format!("✅ {}", status.message)));
                            // Refresh git status after commit
                            api_get_git_status(Some({
                                let git_status_clone = git_status_clone.clone();
                                move |result: Result<GitRepoStatus, String>| {
                                    if let Ok(status) = result {
                                        git_status_clone.set(Some(status));
                                    }
                                }
                            }));
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

    let force_pull_changes = {
        let status_message = status_message.clone();
        let is_loading_force_pull = is_loading_force_pull.clone();
        let git_repo_status = git_repo_status.clone();
        
        Callback::from(move |_| {
            // Show confirmation dialog
            if !web_sys::window()
                .and_then(|w| w.confirm_with_message("⚠️ WARNING: Force pull will OVERWRITE all local changes! Are you sure?").ok())
                .unwrap_or(false) {
                return;
            }
            
            is_loading_force_pull.set(true);
            status_message.set(None);
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading_force_pull.clone();
            let git_status_clone = git_repo_status.clone();
            
            api_force_pull(Some(move |result: Result<GitStatus, String>| {
                is_loading_clone.set(false);
                match result {
                    Ok(status) => {
                        if status.success {
                            status_msg_clone.set(Some(format!("✅ {}", status.message)));
                            // Refresh git status after force pull
                            api_get_git_status(Some({
                                let git_status_clone = git_status_clone.clone();
                                move |result: Result<GitRepoStatus, String>| {
                                    if let Ok(status) = result {
                                        git_status_clone.set(Some(status));
                                    }
                                }
                            }));
                            // Reload page to show updated files
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

    let push_changes = {
        let status_message = status_message.clone();
        let is_loading_push = is_loading_push.clone();
        let git_repo_status = git_repo_status.clone();
        
        Callback::from(move |_| {
            is_loading_push.set(true);
            status_message.set(None);
            
            let status_msg_clone = status_message.clone();
            let is_loading_clone = is_loading_push.clone();
            let git_status_clone = git_repo_status.clone();
            
            api_push_changes(Some(move |result: Result<GitStatus, String>| {
                is_loading_clone.set(false);
                match result {
                    Ok(status) => {
                        if status.success {
                            status_msg_clone.set(Some(format!("✅ {}", status.message)));
                            // Refresh git status after push
                            api_get_git_status(Some({
                                let git_status_clone = git_status_clone.clone();
                                move |result: Result<GitRepoStatus, String>| {
                                    if let Ok(status) = result {
                                        git_status_clone.set(Some(status));
                                    }
                                }
                            }));
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
        <div class="settings-page p-3">
            <header class="mb-4">
                <div class="flex items-center justify-between mb-4 w-full">
                    <div>
                        <h1 class="font-bold text-xl mb-2">{"⚙️ Settings"}</h1>
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
                            { "🚪 Logout" }
                        </button>
                        <Link<Route> to={Route::WebEditor} classes="btn btn-secondary text-sm">
                            { "← Back to Editor" }
                        </Link<Route>>
                    </div>
                </div>
            </header>

            <div class="space-y-6">
                // Git Repository Status
                { if let Some(ref status) = *git_repo_status {
                    if status.success {
                        html! {
                            <section class="bg-card p-4 rounded border">
                                <h2 class="font-bold mb-4">{"📊 Repository Status"}</h2>
                                
                                // Show recent operation status if any
                                { if let Some(ref msg) = *status_message {
                                    html! {
                                        <div class="mb-3 p-2 rounded text-sm" style={
                                            if msg.starts_with("✅") {
                                                "background-color: #dcfce7; color: #166534; border: 1px solid #bbf7d0;"
                                            } else if msg.starts_with("❌") {
                                                "background-color: #fef2f2; color: #dc2626; border: 1px solid #fecaca;"
                                            } else {
                                                "background-color: #fefce8; color: #ca8a04; border: 1px solid #fde68a;"
                                            }
                                        }>
                                            { msg }
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            
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
                                            <div class="flex items-center gap-1 text-orange-600">
                                                <span>{"⬇️"}</span>
                                                <span class="text-sm">{format!("{} behind", status.behind_count)}</span>
                                            </div>
                                        }
                                    } else if status.ahead_count > 0 {
                                        html! {
                                            <div class="flex items-center gap-1 text-blue-600">
                                                <span>{"⬆️"}</span>
                                                <span class="text-sm">{format!("{} ahead", status.ahead_count)}</span>
                                            </div>
                                        }
                                    } else {
                                        html! {
                                            <div class="flex items-center gap-1 text-green-600">
                                                <span>{"✅"}</span>
                                                <span class="text-sm">{"Up to date"}</span>
                                            </div>
                                        }
                                    }}
                                </div>
                                
                                { if status.has_changes {
                                    html! {
                                        <div class="space-y-3 mt-4">
                                            <div class="text-sm font-medium text-orange-600">{"🔄 Local Changes Detected:"}</div>
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
                                        <div class="flex items-center gap-2 text-green-600 mt-4">
                                            <span>{"✅"}</span>
                                            <span class="text-sm">{"Working directory clean"}</span>
                                        </div>
                                    }
                                }}
                                
                                // Actions for changes
                                { if status.has_changes {
                                    html! {
                                        <div class="space-y-3 pt-4 mt-4 border-t">
                                            <div>
                                                <label class="block text-sm font-medium mb-1">
                                                    {"Commit Message"}
                                                </label>
                                                <input 
                                                    type="text"
                                                    class="input w-full"
                                                    placeholder="Updated files via simple_web"
                                                    value={(*commit_message).clone()}
                                                    onchange={on_commit_message_change}
                                                />
                                            </div>
                                            <div class="flex gap-2 flex-wrap">
                                                <button 
                                                    class="btn btn-primary"
                                                    onclick={commit_changes}
                                                    disabled={*is_loading_commit}
                                                >
                                                    { if *is_loading_commit { "⏳ Committing..." } else { "📝 Commit Changes" } }
                                                </button>
                                                <button 
                                                    class="btn btn-danger"
                                                    onclick={force_pull_changes}
                                                    disabled={*is_loading_force_pull}
                                                >
                                                    { if *is_loading_force_pull { "⏳ Force Pulling..." } else { "⚠️ Force Pull (Discard Changes)" } }
                                                </button>
                                            </div>
                                        </div>
                                    }
                                } else if status.ahead_count > 0 {
                                    // We have unpushed commits
                                    html! {
                                        <div class="pt-4 mt-4 border-t space-y-3">
                                            <div class="flex items-center gap-2 text-blue-600">
                                                <span>{"📤"}</span>
                                                <span class="text-sm">{format!("You have {} unpushed commit(s)", status.ahead_count)}</span>
                                            </div>
                                            <div class="flex gap-2 flex-wrap">
                                                <button 
                                                    class="btn btn-primary"
                                                    onclick={push_changes}
                                                    disabled={*is_loading_push}
                                                >
                                                    { if *is_loading_push { "⏳ Pushing..." } else { "📤 Push Changes" } }
                                                </button>
                                                <button 
                                                    class="btn btn-danger"
                                                    onclick={force_pull_changes}
                                                    disabled={*is_loading_force_pull}
                                                >
                                                    { if *is_loading_force_pull { "⏳ Force Pulling..." } else { "⚠️ Force Pull (Discard Changes)" } }
                                                </button>
                                            </div>
                                            { if status.behind_count > 0 {
                                                html! {
                                                    <div class="text-sm text-orange-600 px-2 py-1 bg-orange-50 rounded">
                                                        {"⚠️ Cannot pull: you are both ahead and behind. Push first or use Force Pull."}
                                                    </div>
                                                }
                                            } else {
                                                html! {}
                                            }}
                                        </div>
                                    }
                                } else if status.behind_count > 0 {
                                    // We need to pull changes
                                    html! {
                                        <div class="pt-4 mt-4 border-t">
                                            <button 
                                                class="btn btn-primary"
                                                onclick={pull_updates.clone()}
                                                disabled={*is_loading_pull}
                                            >
                                                { if *is_loading_pull { "⏳ Pulling..." } else { "🔄 Pull Updates" } }
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
                        html! {
                            <section class="bg-card p-4 rounded border">
                                <h2 class="font-bold mb-4">{"📊 Repository Status"}</h2>
                                
                                // Show recent operation status if any
                                { if let Some(ref msg) = *status_message {
                                    html! {
                                        <div class="mb-3 p-2 rounded text-sm" style={
                                            if msg.starts_with("✅") {
                                                "background-color: #dcfce7; color: #166534; border: 1px solid #bbf7d0;"
                                            } else if msg.starts_with("❌") {
                                                "background-color: #fef2f2; color: #dc2626; border: 1px solid #fecaca;"
                                            } else {
                                                "background-color: #fefce8; color: #ca8a04; border: 1px solid #fde68a;"
                                            }
                                        }>
                                            { msg }
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                                
                                <div class="flex items-center gap-2 text-gray-600">
                                    <span>{"⚠️"}</span>
                                    <span class="text-sm">{"No Git repository configured. Please setup a repository below to enable version control."}</span>
                                </div>
                            </section>
                        }
                    }
                } else {
                    html! {
                        <section class="bg-card p-4 rounded border">
                            <h2 class="font-bold mb-4">{"📊 Repository Status"}</h2>
                            
                            // Show recent operation status if any
                            { if let Some(ref msg) = *status_message {
                                html! {
                                    <div class="mb-3 p-2 rounded text-sm" style={
                                        if msg.starts_with("✅") {
                                            "background-color: #dcfce7; color: #166534; border: 1px solid #bbf7d0;"
                                        } else if msg.starts_with("❌") {
                                            "background-color: #fef2f2; color: #dc2626; border: 1px solid #fecaca;"
                                        } else {
                                            "background-color: #fefce8; color: #ca8a04; border: 1px solid #fde68a;"
                                        }
                                    }>
                                        { msg }
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                            
                            <div class="flex items-center gap-2 text-gray-600">
                                <span>{"⏳"}</span>
                                <span class="text-sm">{"Checking repository status..."}</span>
                            </div>
                        </section>
                    }
                }}

                // Git Repository Configuration
                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"🔧 Git Repository Configuration"}</h2>
                    
                    <div>
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
                    
                    <div>
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


                // Editor Theme Configuration
                <section class="bg-card p-4 rounded border">
                    <h2 class="font-bold mb-4">{"🎨 Editor Theme Configuration"}</h2>
                    <ThemeEditor />
                </section>

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