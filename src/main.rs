use axum::{
    Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use serde_json::json;
use sha2::Sha256;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    println!(
        "{}",
        generate_token(dotenvy::var("SECRET").unwrap().as_bytes())
    );
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("0.0.0.0:{}", port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid binding address");
    let serve_dir = ServeDir::new("frontend/dist")
        .not_found_service(ServeFile::new(format!("{}/index.html", "frontend/dist")));

    let app = Router::new()
        // .route("/", get(home_handler))
        // .route("/api/ping", get(ping_handler))
        .route("/ws", get(ws_handler))
        .route("/token/get", get(token_handler))
        .route_service("/test-1", ServeFile::new("frontend/test-page.html"))
        .route_service("/test-2", ServeFile::new("frontend/test-page-2.html"))
        .fallback_service(serve_dir);

    println!("Server starting continuously on {}", addr_str);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

type HmacSha256 = Hmac<Sha256>;
fn generate_token(secret: &[u8]) -> String {
    let mut random = [0u8; 16];
    rand::fill(&mut random);

    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(&random);
    let signature = mac.finalize().into_bytes();

    let mut v = Vec::with_capacity(random.len() + signature.len());
    v.extend_from_slice(&random);
    v.extend_from_slice(&signature);

    URL_SAFE_NO_PAD.encode(v)
}

fn verify_token(token: &str, secret: &[u8]) -> bool {
    let combined = match URL_SAFE_NO_PAD.decode(token) {
        Ok(v) => v,
        Err(_) => return false,
    };

    if combined.len() != 48 {
        return false;
    }

    let (random, signature) = combined.split_at(16);
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(random);

    mac.verify_slice(signature).is_ok()
}

async fn token_handler() -> impl IntoResponse {
    let token = generate_token(b"ok");

    Json(json!({
        "token": token
    }))
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    println!("new connection!!");
    while let Some(result) = socket.recv().await {
        match result {
            Ok(msg) => {
                // if let Message::Text(text) = msg {
                //     if socket.send(Message::Text(text)).await.is_err() {
                //         break;
                //     }
                // }
            }
            Err(_) => break,
        }
    }
}
