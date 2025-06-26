#[macro_use] extern crate rocket;

mod api;
mod auth;

use rocket::{fs::{relative, FileServer}, http::Method};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

#[launch]
fn rocket() -> _ {
    let api_url = std::env::var("API_URL").unwrap_or_default();
    let editor_url = std::env::var("EDITOR_URL").unwrap_or_default();
    let allowed_origins = AllowedOrigins::some_exact(&[
        // local testing
        "http://127.0.0.1:8080",
        "http://localhost:8080",
        "http://127.0.0.1:8000",
        "http://localhost:8000",
        // production
        api_url.as_str(),
        editor_url.as_str(),
    ]);

    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete, Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Content-Type"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Error configuring CORS");

    rocket::build()
        .attach(cors)
        .mount("/api", routes![
            api::list_files,
            api::get_file,
            api::save_file,
            api::delete_file,
            api::move_entry,
            api::upload
        ])
        // Anything under `public_site/` is  always available under /
        .mount("/", FileServer::from(relative!("public_site")))
}