#[macro_use] extern crate rocket;

mod api;
mod auth;

use rocket::fs::{FileServer, relative};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![
            api::list_files,
            api::get_file,
            api::save_file,
            api::delete_file,
            api::move_entry,
            api::upload
        ])
        // Anything under `public_site/` is  always available under /
        .mount("/", FileServer::from(relative!("../public_site")))
}