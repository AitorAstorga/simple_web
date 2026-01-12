// frontend_simple_web/src/file.rs
use gloo::{console::{debug, error, log}, net::{http::{Request, Response}, Error}};
use urlencoding::encode;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{js_sys::Reflect, FileList};

use crate::api::auth::{get_token, handle_auth_error};

// Helper method to reload the page
fn reload() { let _ = web_sys::window().map(|w| w.location().reload()); }

pub fn post_api_file(path: impl Into<String>, content: impl Into<String>) {
    let path    = path.into();
    let content = content.into();
    let auth    = get_token();

    spawn_local(async move {
        let url  = format!("/api/file?path={}", encode(&path));
        let body = serde_json::json!({ "content": content }).to_string();

        let req = match Request::post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(body)
        {
            Ok(r) => r,
            Err(_) => return,
        };

        match req.send().await {
            Ok(response) => {
                if !handle_auth_error(response.status()) {
                    reload();
                }
            }
            Err(_) => {
                error!("Failed to send request");
            }
        }
    });
}

pub async fn get_api_file(path: &str) -> Result<Response, Error> {
    let url = format!("/api/file?path={}", encode(path));
    let auth = get_token();
    
    let response = Request::get(&url)
        .header("Authorization", &auth)
        .send()
        .await?;
    
    // Check for authentication errors
    if handle_auth_error(response.status()) {
        return Err(Error::GlooError(format!("Authentication failed").into()));
    }
    
    Ok(response)
}

pub async fn get_api_files(path: &str) -> Result<Response, Error> {
    let url = format!("/api/files?path={}", encode(path));
    let auth = get_token();
    
    let response = Request::get(&url)
        .header("Authorization", &auth)
        .send()
        .await?;
    
    // Check for authentication errors
    if handle_auth_error(response.status()) {
        return Err(Error::GlooError(format!("Authentication failed").into()));
    }
    
    Ok(response)
}

pub fn api_move(from: impl Into<String>, to: impl Into<String>) {
    let from = from.into();
    let to   = to.into();
    let auth    = get_token();

    log!(format!("moving {from} to {to}"));

    spawn_local(async move {
        let body = serde_json::json!({ "from": &from, "to": &to }).to_string();
        match Request::post("/api/move")
            .header("Authorization", &auth)
            .header("Content-Type", "application/json")
            .body(body)
            .expect("failed to build move-request")
            .send()
            .await {
            Ok(response) => {
                handle_auth_error(response.status());
            }
            Err(_) => {
                error!("Failed to move file");
            }
        }
    });
}

pub fn api_delete(path: impl Into<String>) {
    let path = path.into();
    let auth    = get_token();

    spawn_local(async move {
        let url = format!("/api/file?path={}", encode(&path));
        match Request::delete(&url)
            .header("Authorization", &auth)
            .send()
            .await {
            Ok(response) => {
                if !handle_auth_error(response.status()) {
                    reload();
                }
            }
            Err(_) => {
                error!("Failed to delete file");
            }
        }
    });
}

pub fn api_upload(files: FileList, base_path: Option<String>) {
    let auth    = get_token();

    if files.length() == 0 {
        error!("No files selected");
        return;
    }

    debug!(format!("Uploading {} files", files.length()));

    let form_data = web_sys::FormData::new().expect("should create FormData");

    if let Some(bp) = base_path {
        form_data.append_with_str("base_path", &bp).unwrap();
    }

    for i in 0..files.length() {
        let js_file = files.item(i).unwrap();
        let file: web_sys::File = js_file.unchecked_into();
        // Try to read `webkitRelativePath` via Reflect
        let rel_path = Reflect::get(&file, &JsValue::from_str("webkitRelativePath"))
            .ok()
            .and_then(|v| v.as_string())
            // fallback to just the filename if that property wasn't set
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| file.name());

        // Append the blob with the full path as its filename
        form_data
            .append_with_blob_and_filename("files", &file, &rel_path)
            .unwrap();

        debug!(format!("Uploading as `{}`", rel_path));
    }

    spawn_local(async move {
        let url = "/api/upload".to_string();
        let req = match Request::post(&url)
            .header("Authorization", &auth)
            .body(form_data)
        {
            Ok(r) => r,
            Err(_) => return,
        };
        match req.send().await {
            Ok(response) => {
                if handle_auth_error(response.status()) {
                    error!("Authentication failed during upload");
                } else if !response.ok() {
                    error!("Upload failed with status: {}", response.status());
                }
                // Note: Commented out reload for uploads as it might interrupt user flow
            }
            Err(_) => {
                error!("Failed to upload files");
            }
        }
    });
}