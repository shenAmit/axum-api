use axum::{routing::get, Router};
use crate::handlers::user::get_user;

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(|| async { "API Running 🚀" }))
        .route("/user", get(get_user))
}
