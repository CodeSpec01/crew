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
use tauri::{AppHandle, Emitter};

struct AppState {
    clients: Mutex<HashMap<String, String>>,
    app_handle: AppHandle, // New field to talk to Svelte
}

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

// THIS MACRO IS THE KEY TO ANDROID
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            // Clone the handle before moving it into the spawned thread
            let app_handle = _app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let state = Arc::new(AppState {
                    clients: Mutex::new(HashMap::new()),
                    app_handle,
                });

                let app = Router::new()
                    .route("/", get(index_handler))
                    .route("/ws", get(ws_handler))
                    .layer(Extension(state.clone()));

                let _ = match local_ip() {
                    Ok(IpAddr::V4(ipv4)) => ipv4.to_string(),
                    _ => "127.0.0.1".to_string(),
                };

                let listener = match tokio::net::TcpListener::bind("0.0.0.0:6769").await {
                    Ok(l) => l,
                    Err(_) => {
                        tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap()
                    }
                };
                
                let port = listener.local_addr().unwrap().port();

                // Fire the first log to the UI!
                let _ = state.app_handle.emit("server-log", format!("🚀 SERVER RUNNING ON PORT {}", port));
                
                println!("🚀 BARE-METAL SERVER RUNNING ON PORT {}", port);

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

        // Emit Join Event
        let _ = state.app_handle.emit("server-log", format!("🟢 JOINED: [{}] from {}", socket_id, ip));
        // Create a JSON array of all connected peers and broadcast it
        let peer_list: Vec<serde_json::Value> = clients.iter().map(|(id, peer_ip)| {
            serde_json::json!({ "id": id, "ip": peer_ip })
        }).collect();
        let _ = state.app_handle.emit("peer-update", peer_list);
    }

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
                    // Emit Drop Event
                    let _ = state.app_handle.emit("server-log", format!("⚠️ NETWORK DROP: [{}]", socket_id));
                    break;
                }
            }
        }
    }

    {
        let mut clients = state.clients.lock().unwrap();
        clients.remove(&socket_id);

        // Emit Leave Event
        let _ = state.app_handle.emit("server-log", format!("🔴 LEFT: [{}]", socket_id));
        // Broadcast the updated list after someone leaves
        let peer_list: Vec<serde_json::Value> = clients.iter().map(|(id, peer_ip)| {
            serde_json::json!({ "id": id, "ip": peer_ip })
        }).collect();
        let _ = state.app_handle.emit("peer-update", peer_list);
    }
}