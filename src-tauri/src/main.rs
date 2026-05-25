use axum::{routing::get, Router};
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;
use local_ip_address::local_ip;
use tokio::sync::oneshot; // <-- Import the oneshot channel

struct AppState {
    room_code: Mutex<Option<String>>,
    // We add our kill switch sender here
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
}

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

#[tauri::command]
async fn start_host_session(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let code = generate_shout_code();
    let host_ip = get_local_ip();
    
    // 1. Create a oneshot channel for our kill switch
    let (tx, rx) = oneshot::channel();

    // 2. Lock state and store the sender (tx)
    {
        let mut room_code = state.room_code.lock().unwrap();
        *room_code = Some(code.clone());

        let mut shutdown_tx = state.shutdown_tx.lock().unwrap();
        // If a server is somehow already running, kill it first to prevent port collisions
        if let Some(old_tx) = shutdown_tx.take() {
            let _ = old_tx.send(()); 
        }
        *shutdown_tx = Some(tx); // Store the new sender
    }

    println!("Host session started! Code: {} | IP: {}", code, host_ip);

    let app = Router::new().route("/", get(|| async { "Crew Local Server is Running!" }));
    let addr: SocketAddr = format!("0.0.0.0:3000").parse().unwrap();
    
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        println!("Listening for crew members on port 3000...");
        
        // 3. Attach the receiver (rx) to Axum's graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                rx.await.ok(); // Wait here until the kill signal is received
                println!("Server shutting down gracefully... Port 3000 freed.");
            })
            .await
            .unwrap();
    });

    Ok(format!("{}|{}", code, host_ip))
}

// 4. Create the new command to trigger the kill switch
#[tauri::command]
fn stop_host_session(state: tauri::State<'_, AppState>) {
    let mut room_code = state.room_code.lock().unwrap();
    *room_code = None;

    let mut shutdown_tx = state.shutdown_tx.lock().unwrap();
    // Take the sender out of state and fire it
    if let Some(tx) = shutdown_tx.take() {
        let _ = tx.send(());
        println!("Kill signal sent to server.");
    }
}

fn main() {
    let app_state = AppState {
        room_code: Mutex::new(None),
        shutdown_tx: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(app_state)
        // 5. Register the new command here!
        .invoke_handler(tauri::generate_handler![start_host_session, stop_host_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}