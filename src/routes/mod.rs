use axum::{routing::get, Router};
use crate::handlers::user::get_user;
use crate::handlers::ws::ws_handler;

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(|| async { "API Running 🚀" }))
        .route("/user", get(get_user))
        .route("/ws", get(ws_handler))
}
