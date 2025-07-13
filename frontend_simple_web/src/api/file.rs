// frontend_simple_web/src/file.rs
use gloo::{console::log, net::{http::{Request, Response}, Error}};
use urlencoding::encode;
use wasm_bindgen_futures::spawn_local;

use crate::{api::auth::get_token, config_file::get_env_var};

// Helper method to reload the page
fn reload() { let _ = web_sys::window().map(|w| w.location().reload()); }

pub fn post_api_file(path: impl Into<String>, content: impl Into<String>) {
    let path    = path.into();
    let content = content.into();
    let api_url = get_env_var("API_URL");
    let auth    = get_token();

    spawn_local(async move {
        let url  = format!("{api_url}/api/file?path={}", encode(&path));
        let body = serde_json::json!({ "content": content }).to_string();

        let req = match Request::post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(body)
        {
            Ok(r) => r,
            Err(_) => return,
        };

        let _ = req.send().await;
        reload();
    });
}

pub async fn get_api_file(path: &str) -> Result<Response, Error> {
    let api_url = get_env_var("API_URL");
    let url = format!("{api_url}/api/file?path={}", encode(path));
    let auth = get_token();
    Request::get(&url)
        .header("Authorization", &auth)
        .send()
        .await
}

pub async fn get_api_files(path: &str) -> Result<Response, Error> {
    let api_url = get_env_var("API_URL");
    let url = format!("{api_url}/api/files?path={}", encode(path));
    let auth = get_token();
    Request::get(&url)
        .header("Authorization", &auth)
        .send()
        .await
}

pub fn api_move(from: impl Into<String>, to: impl Into<String>) {
    let from = from.into();
    let to   = to.into();
    let api_url = get_env_var("API_URL");
    let auth    = get_token();

    log!(format!("moving {from} to {to}"));

    spawn_local(async move {
        let body = serde_json::json!({ "from": &from, "to": &to }).to_string();
        let _ = Request::post(&format!("{api_url}/api/move"))
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(body)
            .expect("failed to build move-request")
            .send()
            .await;
    });
}

pub fn api_delete(path: impl Into<String>) {
    let path = path.into();
    let api_url = get_env_var("API_URL");
    let auth    = get_token();

    spawn_local(async move {
        let url = format!("{api_url}/api/file?path={}", encode(&path));
        let _ = Request::delete(&url)
            .header("Authorization", &auth)
            .send()
            .await;
        reload();
    });
}