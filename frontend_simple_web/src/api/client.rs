// frontend_simple_web/src/api/client.rs
//
// Unified API client — all HTTP requests go through here.

use gloo::net::http::Request;
use serde::de::DeserializeOwned;
use wasm_bindgen_futures::spawn_local;

use super::auth::{get_token, handle_auth_error};

pub enum Method {
    Get,
    Post,
    Delete,
}

/// Low-level async request. Returns the parsed JSON body or an error string.
/// Use this when you are already inside an async context and need the result.
pub async fn request<T: DeserializeOwned>(method: Method, url: &str, body: Option<String>) -> Result<T, String> {
    let auth = get_token();

    let req = match method {
        Method::Get => Request::get(url)
            .header("Authorization", &auth)
            .build(),
        Method::Post => match body {
            Some(ref b) => Request::post(url)
                .header("Authorization", &auth)
                .header("Content-Type", "application/json")
                .body(b.as_str()),
            None => Request::post(url)
                .header("Authorization", &auth)
                .build(),
        },
        Method::Delete => Request::delete(url)
            .header("Authorization", &auth)
            .build(),
    };

    let req = req.map_err(|e| format!("Failed to build request: {:?}", e))?;

    let response = req
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if handle_auth_error(response.status()) {
        return Err("Authentication failed".to_string());
    }

    response
        .json::<T>()
        .await
        .map_err(|e| format!("Failed to parse response: {:?}", e))
}

/// Fire-and-forget wrapper around `request`.
/// Spawns a local task to make the request, then calls the callback with the result.
pub fn spawn_request<T, F>(method: Method, url: String, body: Option<String>, callback: Option<F>)
where
    T: DeserializeOwned + 'static,
    F: Fn(Result<T, String>) + 'static,
{
    spawn_local(async move {
        let result = request::<T>(method, &url, body).await;
        if let Some(cb) = callback {
            cb(result);
        }
    });
}
