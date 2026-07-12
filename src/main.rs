use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // 1. Read Render's assigned port env variable, default to 3000 locally
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("0.0.0.0:{}", port); // Must bind to 0.0.0.0 on Render
    let addr: SocketAddr = addr_str.parse().expect("Invalid binding address");

    // 2. Map all endpoints to a single unified application router
    let app = Router::new()
        // Serve the interactive test HTML panel at the root route
        .route("/", get(home_handler))
        // Ultra-lightweight endpoint for UptimeRobot to keep the service awake
        .route("/api/ping", get(ping_handler))
        // Persistent bidirectional WebSocket connection pipe
        .route("/ws", get(ws_handler));

    println!("🚀 Server starting continuously on {}", addr_str);

    // 3. Bind the server to the TCP listener loop
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler that instantly returns HTTP 200 OK "pong" text to satisfy monitoring tools
async fn ping_handler() -> &'static str {
    "pong"
}

// Handler that serves a visual dashboard containing real-time client-side WebSocket code
async fn home_handler() -> Html<&'static str> {
    Html(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Axum Echo System</title>
        <style>
            body { font-family: monospace; background: #121214; color: #e1e1e6; padding: 40px; }
            #output { border: 1px solid #323238; padding: 15px; height: 250px; overflow-y: auto; background: #1a1a1e; }
            input, button { padding: 10px; background: #323238; color: #fff; border: 1px solid #48484a; font-family: inherit; }
            button { background: #04d361; color: #000; font-weight: bold; cursor: pointer; }
        </style>
    </head>
    <body>
        <h2>Axum WebSocket Echo Server</h2>
        <div id="output">Connecting to WebSocket stream...<br></div>
        <br>
        <input type="text" id="messageInput" placeholder="Type a message..." value="Hello from browser!">
        <button onclick="sendMessage()">Send Echo</button>

        <script>
            // Automatically determine protocol (ws:// vs wss://) based on standard deployment status
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUri = `${protocol}//${window.location.host}/ws`;

            let socket;

            function connect() {
                socket = new WebSocket(wsUri);

                socket.onopen = () => {
                    log("✅ Connected securely to WebSocket endpoint");
                };

                socket.onmessage = (event) => {
                    log(`📥 Echo Received: ${event.data}`);
                };

                socket.onclose = () => {
                    log("❌ Connection severed. Attempting auto-reconnect fallback loop...");
                    setTimeout(connect, 3000); // Guarding against Render deployment resets
                };
            }

            function sendMessage() {
                const input = document.getElementById("messageInput");
                if (socket && socket.readyState === WebSocket.OPEN) {
                    log(`📤 Sent: ${input.value}`);
                    socket.send(input.value);
                } else {
                    log("⚠️ Cannot dispatch: stream not active");
                }
            }

            function log(msg) {
                const output = document.getElementById("output");
                output.innerHTML += msg + "<br>";
                output.scrollTop = output.scrollHeight;
            }

            connect();
        </script>
    </body>
    </html>
    "#,
    )
}

// Extractor that evaluates standard HTTP headers before completing the upgrade handshake
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

// Thread loop managing individual client read/write streaming operations
async fn handle_socket(mut socket: WebSocket) {
    while let Some(result) = socket.recv().await {
        match result {
            Ok(msg) => {
                // Intercept Text messages and mirror them straight back to the client channel
                if let Message::Text(text) = msg {
                    if socket.send(Message::Text(text)).await.is_err() {
                        break; // Connection severed by client early
                    }
                }
            }
            Err(_) => break, // Catch exceptions/disconnections and drop thread cleanly
        }
    }
}
