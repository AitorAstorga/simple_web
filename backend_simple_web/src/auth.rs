use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;

/// Simple bearer‑token guard – set `ADMIN_TOKEN` env‑var before starting the server
pub struct Admin;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = std::env::var("ADMIN_TOKEN").unwrap_or_default();
        match req.headers().get_one("Authorization") {
            Some(t) if t == token => Outcome::Success(Admin),
            _ => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}