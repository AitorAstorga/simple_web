use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use uuid::Uuid;

const TOKEN_TTL: Duration = Duration::from_secs(60 * 10); // 10 minutes

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Login {
    pub password: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TokenResponse {
    pub token: String,
}

pub struct TokenStore(RwLock<HashMap<String, Instant>>);

impl TokenStore {
    pub fn new() -> Self {
        TokenStore(RwLock::new(HashMap::new()))
    }

    /// Insert a token with timestamp `now`
    pub fn insert(&self, token: String) {
        self.0.write().unwrap().insert(token, Instant::now());
    }

    /// Check if a token is valid and hasn't expired
    pub fn validate(&self, token: &str) -> bool {
        let mut map = self.0.write().unwrap();
        if let Some(&t0) = map.get(token) {
            if t0.elapsed() <= TOKEN_TTL {
                return true;
            } else {
                // Token expired
                map.remove(token);
            }
        }
        false
    }
}

/// Login and return a token
/// ### Arguments
/// - `body`: JSON with the password
/// ### Returns
/// - `Json<TokenResponse>`
/// ### Examples
/// - `curl -i -X POST -H "Content-Type: application/json" -d '{"password":"secret123"}' http://localhost:8000/api/login`
#[rocket::post("/", format = "json", data = "<body>")]
pub fn login(body: Json<Login>, store: &State<TokenStore>) -> Result<Json<TokenResponse>, Status> {
    let expected = std::env::var("ADMIN_PASSWORD").unwrap_or_default();
    if body.password == expected {
        let token = Uuid::new_v4().to_string();
        store.insert(token.clone());
        Ok(Json(TokenResponse { token }))
    } else {
        Err(Status::Unauthorized)
    }
}

/// Guard to check if the request has a valid token
pub struct Admin;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, ()> {
        let store = req.guard::<&State<TokenStore>>().await.unwrap();
        match req.headers().get_one("Authorization") {
            Some(t) if store.validate(t) => Outcome::Success(Admin),
            _ => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}