use axum::Router;
use dotenvy::dotenv;
use std::env;
use sqlx::mysql::MySqlPoolOptions;
use tracing_subscriber;

mod routes;
mod handlers;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL missing");

    let port = env::var("PORT").unwrap_or("3000".to_string());

    let _db = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("DB connection failed");

    println!("✅ Connected to MySQL");

    let app = Router::new()
        .merge(routes::create_routes());

    // Bind to all interfaces so it works inside Docker containers.
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    println!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}