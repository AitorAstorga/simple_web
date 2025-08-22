// frontend_simple_web/src/api/utils.rs
use gloo::console::error;
use gloo::net::http::Response;
use serde::de::DeserializeOwned;
use crate::api::auth::handle_auth_error;

/// Generic API response handler that checks auth and parses JSON
pub async fn handle_api_response<T, F>(
    response_result: Result<Response, gloo::net::Error>,
    callback: Option<F>,
    operation_name: &str,
) -> Option<T>
where
    T: DeserializeOwned + Clone,
    F: FnOnce(Result<T, String>),
{
    match response_result {
        Ok(response) => {
            // Check for authentication errors first
            if handle_auth_error(response.status()) {
                if let Some(cb) = callback {
                    cb(Err("Authentication failed".to_string()));
                }
                return None;
            }

            // Parse JSON response
            match response.json::<T>().await {
                Ok(data) => {
                    if let Some(cb) = callback {
                        cb(Ok(data.clone()));
                    }
                    Some(data)
                }
                Err(e) => {
                    let error_msg = format!("Failed to parse {} response: {:?}", operation_name, e);
                    error!(&error_msg);
                    if let Some(cb) = callback {
                        cb(Err(format!("Failed to parse response")));
                    }
                    None
                }
            }
        }
        Err(e) => {
            let error_msg = format!("{} request failed: {:?}", operation_name, e);
            error!(&error_msg);
            if let Some(cb) = callback {
                cb(Err("Request failed".to_string()));
            }
            None
        }
    }
}

/// Helper for API calls that don't return data but might trigger page reload
pub async fn handle_api_action_response<F>(
    response_result: Result<Response, gloo::net::Error>,
    callback: Option<F>,
    operation_name: &str,
    should_reload: bool,
) where
    F: FnOnce(Result<(), String>),
{
    match response_result {
        Ok(response) => {
            // Check for authentication errors first
            if handle_auth_error(response.status()) {
                if let Some(cb) = callback {
                    cb(Err("Authentication failed".to_string()));
                }
                return;
            }

            if response.ok() {
                if let Some(cb) = callback {
                    cb(Ok(()));
                } else if should_reload {
                    // Helper method to reload the page
                    let _ = web_sys::window().map(|w| w.location().reload());
                }
            } else {
                let error_msg = format!("{} failed with status: {}", operation_name, response.status());
                error!(&error_msg);
                if let Some(cb) = callback {
                    cb(Err(error_msg));
                }
            }
        }
        Err(e) => {
            let error_msg = format!("{} request failed: {:?}", operation_name, e);
            error!(&error_msg);
            if let Some(cb) = callback {
                cb(Err("Request failed".to_string()));
            }
        }
    }
}

/// Create authenticated request headers
pub fn create_auth_headers() -> Vec<(&'static str, String)> {
    let token = crate::api::auth::get_token();
    vec![("Authorization", token)]
}

/// Add authentication header to a request builder
pub fn add_auth_header(request: gloo::net::http::RequestBuilder) -> gloo::net::http::RequestBuilder {
    let token = crate::api::auth::get_token();
    request.header("Authorization", &token)
}