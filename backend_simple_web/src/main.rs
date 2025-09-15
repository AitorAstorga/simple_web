// backend_simple_web/src/main.rs
#[macro_use] extern crate rocket;

mod api;
mod scheduler;

use rocket::{fs::FileServer, http::Method, serde::json::Json, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use std::{fs, path::Path};
use serde::Serialize;

#[derive(Serialize)]
struct FrontendConfig {
    api_url: String,
    editor_url: String,
}

fn write_frontend_config(api_url: &str, editor_url: &str) -> std::io::Result<()> {
    let config_dir = Path::new("/usr/share/nginx/html/config");
    fs::create_dir_all(&config_dir)?;
    let config = FrontendConfig {
        api_url: api_url.to_string(),
        editor_url: editor_url.to_string(),
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(config_dir.join("config.json"), json)?;
    Ok(())
}

#[rocket::post("/", format = "json", data = "<body>")]
fn login(body: Json<prisma_auth::LoginRequest>, store: &State<prisma_auth::backend::TokenStore>) -> Result<Json<prisma_auth::TokenResponse>, rocket::http::Status> {
    prisma_auth::backend::login_handler(body, store)
}

#[launch]
async fn rocket() -> _ {
    let api_url = std::env::var("API_URL").expect("Please set API_URL to something like \"https://api.example.com\"");
    let editor_url = std::env::var("EDITOR_URL").expect("Please set EDITOR_URL to something like \"https://editor.example.com\"");

    write_frontend_config(&api_url, &editor_url).expect("Failed to write frontend config");

    // Initialize the git scheduler
    scheduler::init_scheduler().await.expect("Failed to initialize git scheduler");

    // Initialize token store for authentication
    let token_store = prisma_auth::backend::TokenStore::new();

    let allowed_origins = AllowedOrigins::some_exact(&[
        // local SPA on port 80
        "http://127.0.0.1",
        "http://localhost",
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
        .manage(token_store)
        .mount("/api/auth", routes![
            login
        ])
        .mount("/api", routes![
            api::list_files,
            api::get_file,
            api::save_file,
            api::delete_file,
            api::move_entry,
            api::upload_files,
            api::setup_git_repo,
            api::pull_repo,
            api::test_git_repo,
            api::get_auto_pull_config,
            api::set_auto_pull_config,
            api::get_git_status,
            api::commit_changes,
            api::push_repo,
            api::force_pull_repo,
            api::list_themes,
            api::get_theme,
            api::save_theme,
            api::delete_theme
        ])
        // Anything under `public_site/` is  always available under /
        .mount("/", FileServer::from("/public_site"))
}