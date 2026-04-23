// frontend_simple_web/src/hooks/use_git_settings.rs
//
// Consolidates the 4 git-form state handles + their localStorage loading.

use yew::prelude::*;
use crate::api::git::GitRepoConfig;

pub struct GitSettings {
    pub repo_url: UseStateHandle<String>,
    pub branch: UseStateHandle<String>,
    pub username: UseStateHandle<String>,
    pub token: UseStateHandle<String>,
}

impl GitSettings {
    /// Build a `GitRepoConfig` from the current state values.
    pub fn to_config(&self) -> GitRepoConfig {
        GitRepoConfig {
            url: self.repo_url.trim().to_string(),
            branch: if self.branch.trim().is_empty() { None } else { Some(self.branch.trim().to_string()) },
            username: if self.username.trim().is_empty() { None } else { Some(self.username.trim().to_string()) },
            token: if self.token.trim().is_empty() { None } else { Some(self.token.trim().to_string()) },
        }
    }
}

/// Hook that creates 4 state handles for git settings and loads their
/// initial values from localStorage on first render.
#[hook]
pub fn use_git_settings() -> GitSettings {
    let repo_url = use_state(|| String::new());
    let branch = use_state(|| String::new());
    let username = use_state(|| String::new());
    let token = use_state(|| String::new());

    {
        let repo_url = repo_url.clone();
        let branch = branch.clone();
        let username = username.clone();
        let token = token.clone();
        use_effect_with((), move |_| {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(url)) = storage.get_item("git_repo_url") { repo_url.set(url); }
                    if let Ok(Some(br)) = storage.get_item("git_branch") { branch.set(br); }
                    if let Ok(Some(user)) = storage.get_item("git_username") { username.set(user); }
                    if let Ok(Some(tok)) = storage.get_item("git_token") { token.set(tok); }
                }
            }
            || ()
        });
    }

    GitSettings { repo_url, branch, username, token }
}
