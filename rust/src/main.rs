mod engine;
mod handlers;
mod models;
mod repository;

use axum::{routing::post, routing::get, Router};
use std::sync::Arc;
use std::net::SocketAddr;
use mongodb::{Client, options::ClientOptions};
use dotenvy::dotenv;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub db: mongodb::Database,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Загружаем переменные из .env
    dotenv().ok();

    let mongo_uri = std::env::var("MONGO_URI")
        .expect("MONGO_URI must be set in .env or environment");
    let db_name = std::env::var("DATABASE_NAME")
        .unwrap_or_else(|_| "sweet_bananza".to_string());

    let client_options = ClientOptions::parse(mongo_uri).await?;
    let client = Client::with_options(client_options)?;
    let db = client.database(&db_name);

    // Оборачиваем состояние в Arc для безопасного использования между потоками
    let shared_state = Arc::new(AppState { db });

    let cors = CorsLayer::new()
        .allow_origin(Any) // Разрешает запросы с любого домена (для разработки)
        .allow_methods(Any) // Разрешает любые методы (GET, POST, OPTIONS и т.д.)
        .allow_headers(Any); // Разрешает любые заголовки (Content-Type и т.д.)

    let app = Router::new()
        .route("/", get(|| async { "Sweet Bananza API is running!" }))
        .route("/api/spin", post(handlers::game::spin_handler))
        .route("/api/buy-bonus", post(handlers::game::buy_bonus_handler))
        .layer(cors) // ВАЖНО: добавить слой CORS здесь
        .with_state(shared_state);

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    println!("🚀 Server started at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}