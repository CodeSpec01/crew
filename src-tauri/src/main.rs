use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use futures::stream::StreamExt;
use local_ip_address::local_ip;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// --- SHARED STATE ---
struct AppState {
    clients: Mutex<HashMap<String, String>>,
}

// --- DIAGNOSTIC UI (Served directly by Axum) ---
const DIAGNOSTIC_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Crew Node Diagnostic</title>
    <style>
        body { background: #090A0F; color: #00F0FF; font-family: monospace; padding: 20px; }
        #status { font-size: 1.2rem; font-weight: bold; margin-bottom: 20px; }
        #log { border: 1px solid #00F0FF; padding: 10px; height: 60vh; overflow-y: auto; background: rgba(0, 240, 255, 0.05); }
        .timestamp { color: #888; }
    </style>
</head>
<body>
    <h2>Crew Web Node</h2>
    <div id="status">Status: 🟡 Connecting...</div>
    <div id="log"></div>

    <script>
        const logDiv = document.getElementById('log');
        const statusDiv = document.getElementById('status');

        function appendLog(msg) {
            const time = new Date().toLocaleTimeString();
            logDiv.innerHTML += `<div><span class="timestamp">[${time}]</span> ${msg}</div>`;
            logDiv.scrollTop = logDiv.scrollHeight;
        }

        function connect() {
            // Automatically uses the exact IP and Port the browser is visiting
            const wsUrl = `ws://${window.location.host}/ws`;
            appendLog(`Attempting connection to ${wsUrl}...`);
            const ws = new WebSocket(wsUrl);

            ws.onopen = () => {
                statusDiv.innerHTML = 'Status: 🟢 Connected';
                appendLog('Handshake successful. Link established.');
            };

            ws.onmessage = (event) => {
                appendLog(`Host says: ${event.data}`);
            };

            ws.onclose = () => {
                statusDiv.innerHTML = 'Status: 🔴 Disconnected';
                appendLog('Connection lost. Auto-reconnecting in 3 seconds...');
                // The Auto-Reconnect Fallback
                setTimeout(connect, 3000); 
            };

            ws.onerror = () => {
                appendLog('⚠️ Network stream error.');
            };
        }

        connect();
    </script>
</body>
</html>
"#;

async fn index_handler() -> Html<&'static str> {
    Html(DIAGNOSTIC_HTML)
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                let state = Arc::new(AppState {
                    clients: Mutex::new(HashMap::new()),
                });

                // Map both routes: "/" for the UI, "/ws" for the connection
                let app = Router::new()
                    .route("/", get(index_handler))
                    .route("/ws", get(ws_handler))
                    .layer(Extension(state));

                let local_ip = match local_ip() {
                    Ok(IpAddr::V4(ipv4)) => ipv4.to_string(),
                    _ => "127.0.0.1".to_string(),
                };

                // Dynamic Port Binding (Target: 6769)
                let listener = match tokio::net::TcpListener::bind("0.0.0.0:6769").await {
                    Ok(l) => l,
                    Err(_) => {
                        println!("Port 6769 in use, falling back to random free port...");
                        tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap()
                    }
                };
                
                let port = listener.local_addr().unwrap().port();

                println!("=========================================");
                println!("🚀 BARE-METAL SERVER RUNNING");
                println!("📱 Connect your phone to: http://{}:{}", local_ip, port);
                println!("=========================================\n");

                axum::serve(
                    listener,
                    app.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .await
                .unwrap();
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let client_ip = addr.ip().to_string();
    ws.on_upgrade(move |socket| handle_socket(socket, state, client_ip))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>, ip: String) {
    let socket_id = Uuid::new_v4().to_string()[..8].to_string();

    {
        let mut clients = state.clients.lock().unwrap();
        clients.insert(socket_id.clone(), ip.clone());
        println!("🟢 JOINED: [{}] from {}", socket_id, ip);
        println!("👥 ACTIVE: {:?}", clients.keys().collect::<Vec<_>>());
        println!("-----------------------------------------");
    }

    // Adjusted Heartbeat to 10 seconds
    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(10));

    loop {
        tokio::select! {
            msg = socket.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    Some(Ok(_)) => {} 
                }
            }
            _ = ping_interval.tick() => {
                if socket.send(axum::extract::ws::Message::Ping(vec![].into())).await.is_err() {
                    println!("⚠️ Network drop detected for [{}]", socket_id);
                    break;
                }
            }
        }
    }

    {
        let mut clients = state.clients.lock().unwrap();
        clients.remove(&socket_id);
        println!("🔴 LEFT: [{}]", socket_id);
        println!("👥 ACTIVE: {:?}", clients.keys().collect::<Vec<_>>());
        println!("-----------------------------------------");
    }
}