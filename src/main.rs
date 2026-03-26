use axum::Router;
use axum::Extension;
use dotenvy::dotenv;
use std::env;
use sqlx::mysql::MySqlPoolOptions;
use tracing_subscriber;

mod routes;
mod handlers;
mod ws;
mod realtime;
mod redis_client;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://127.0.0.1:6379".to_string());

    let port = env::var("PORT").unwrap_or("3000".to_string());

    let db = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("DB connection failed");

    println!("✅ Connected to MySQL");

    // Run SQL migrations on startup.
    // Requires `sqlx` feature: `migrate` and `./migrations` folder.
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("DB migrations failed");

    let redis = redis_client::connect_redis(&redis_url)
        .await
        .expect("Redis connection failed");
    let redis_client = redis::Client::open(redis_url).expect("Redis client init failed");

    // Lightweight instance id for presence/debugging.
    let server_id = format!("axum-api-{}", std::process::id());
    let rt = realtime::Realtime {
        server_id,
        redis,
        redis_client,
    };

    let app = Router::new()
        .merge(routes::create_routes())
        .layer(Extension(rt));

    // Bind to all interfaces so it works inside Docker containers.
    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();

    println!("🚀 Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}