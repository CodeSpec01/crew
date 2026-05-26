use axum::extract::ConnectInfo;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::sync::oneshot;

// --- PROTOCOL DEFINITIONS ---
#[derive(Serialize, Deserialize, Debug)]
struct WsEvent {
    #[serde(rename = "type")]
    event_type: String,
    payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct JoinPayload {
    name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Peer {
    name: String,
    ip: String,
    device_type: String,
}

// --- APP STATE ---
struct AppState {
    room_code: Mutex<Option<String>>,
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
    peers: Arc<Mutex<HashMap<String, Peer>>>, // Upgraded to Peer
}

#[derive(Clone)]
struct AxumState {
    app_handle: tauri::AppHandle,
    peers: Arc<Mutex<HashMap<String, Peer>>>, // Upgraded to Peer
}

// --- HELPER FUNCTIONS ---
fn get_local_ip() -> String {
    match local_ip() {
        Ok(IpAddr::V4(ipv4)) => ipv4.to_string(),
        Ok(IpAddr::V6(_)) => String::from("IPv6 not supported"),
        Err(_) => String::from("127.0.0.1"),
    }
}

fn generate_shout_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// --- TAURI COMMANDS ---
#[tauri::command]
async fn start_host_session(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle, // Inject Tauri's AppHandle
) -> Result<String, String> {
    let code = generate_shout_code();
    let host_ip = get_local_ip();

    let (tx, rx) = oneshot::channel();

    // Lock and update state
    {
        let mut room_code = state.room_code.lock().unwrap();
        *room_code = Some(code.clone());

        let mut shutdown_tx = state.shutdown_tx.lock().unwrap();
        if let Some(old_tx) = shutdown_tx.take() {
            let _ = old_tx.send(());
        }
        *shutdown_tx = Some(tx);
    }

    println!("Host session started! Code: {} | IP: {}", code, host_ip);

    // Build the specific state for Axum
    let axum_state = AxumState {
        app_handle,
        peers: state.peers.clone(),
    };

    // Build the Axum router with the WS route and injected state
    let app = Router::new()
        .route("/", get(|| async { "Crew Local Server is Running!" }))
        .route("/ws", get(ws_handler))
        .layer(Extension(axum_state));

    // 1. Bind the listener on the main thread first
    let listener_result = tokio::net::TcpListener::bind("0.0.0.0:6739").await;
    let listener = match listener_result {
        Ok(l) => l,
        Err(_) => {
            println!("Port 6739 in use, letting the OS pick a random free port.");
            tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap()
        }
    };

    // 2. Extract the actual assigned port
    let assigned_port = listener.local_addr().unwrap().port();
    println!("Listening for crew members on port {}...", assigned_port);

    // 3. Hand the bound listener to the background task
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            rx.await.ok();
            println!("Server shutting down gracefully... Port freed.");
        })
        .await
        .unwrap();
    });

    // 4. Return Code, IP, AND the dynamic Port to Svelte
    Ok(format!("{}|{}|{}", code, host_ip, assigned_port))
}

#[tauri::command]
fn stop_host_session(state: tauri::State<'_, Arc<AppState>>) {
    let mut room_code = state.room_code.lock().unwrap();
    *room_code = None;

    // Clear connected peers when stopping
    let mut peers = state.peers.lock().unwrap();
    peers.clear();

    let mut shutdown_tx = state.shutdown_tx.lock().unwrap();
    if let Some(tx) = shutdown_tx.take() {
        let _ = tx.send(());
        println!("Kill signal sent to server.");
    }
}

// --- WEBSOCKET LOGIC ---
async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<AxumState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>, // EXTRACT THE IP HERE
) -> impl IntoResponse {
    let client_ip = addr.ip().to_string();
    println!("New connection from IP: {}", client_ip);

    ws.on_upgrade(move |socket| handle_socket(socket, state, client_ip))
}

async fn handle_socket(mut socket: WebSocket, state: AxumState, client_ip: String) {
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                match event.event_type.as_str() {
                    "JOIN_REQUEST" => {
                        if let Ok(payload) = serde_json::from_value::<JoinPayload>(event.payload) {
                            println!("✅ Guest joined: {} from {}", payload.name, client_ip);

                            {
                                let mut peers = state.peers.lock().unwrap();
                                let random_id = format!("guest_{}", rand::random::<u16>());

                                // Create the actual Peer struct
                                let new_peer = Peer {
                                    name: payload.name.clone(),
                                    ip: client_ip.clone(),
                                    device_type: "laptop_mac".to_string(), // We can make this dynamic later
                                };

                                peers.insert(random_id.clone(), new_peer);

                                // Map the structs into a JSON array for Svelte
                                let mut payload_array = Vec::new();
                                for (id, peer) in peers.iter() {
                                    payload_array.push(serde_json::json!({
                                        "id": id,
                                        "name": peer.name,
                                        "ip": peer.ip,
                                        "type": peer.device_type
                                    }));
                                }

                                let _ = state.app_handle.emit("peer-update", payload_array);
                            }

                            // 2. Respond to the guest
                            let response = WsEvent {
                                event_type: "JOIN_ACCEPT".to_string(),
                                payload: serde_json::json!({ "status": "success", "room": "crew-session" }),
                            };
                            let response_text = serde_json::to_string(&response).unwrap();
                            let _ = socket.send(Message::Text(response_text.into())).await;
                        }
                    }
                    _ => println!("Unknown event type: {}", event.event_type),
                }
            }
        }
    }

    // TODO: We will add the DISCONNECT logic here later to remove peers!
    println!("Guest disconnected.");
}

// --- MAIN ENTRANCE ---
fn main() {
    // Initialize state wrapped in Arc right from the start
    let app_state = Arc::new(AppState {
        room_code: Mutex::new(None),
        shutdown_tx: Mutex::new(None),
        peers: Arc::new(Mutex::new(HashMap::new())),
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_host_session,
            stop_host_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
