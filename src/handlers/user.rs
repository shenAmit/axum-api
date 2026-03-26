use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    id: u32,
    name: String,
}

pub async fn get_user() -> Json<User> {
    let user = User {
        id: 1,
        name: "Kishan".to_string(),
    };

    Json(user)
}