use std::sync::{Arc, Mutex};
use tauri::State;
use rand::Rng;

// 1. Define our State
struct AppState {
    room_code: Mutex<Option<String>>,
    // We will add the list of connected peers here later
}

// 2. Create the Tauri Command to start the room
#[tauri::command]
fn start_host_session(state: State<AppState>) -> String {
    // Generate a random 4-character code (e.g., A7K9)
    let code = generate_shout_code(); 
    
    // Lock the mutex and update our state
    let mut room_code = state.room_code.lock().unwrap();
    *room_code = Some(code.clone());

    println!("Host session started with code: {}", code);
    
    // TODO: Later, we will spawn our Axum WebSocket server here

    // Return the code to the Svelte frontend
    code
}

// Helper to generate a code
fn generate_shout_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn main() {
    // 3. Initialize the state
    let app_state = AppState {
        room_code: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(app_state) // Give Tauri the state
        .invoke_handler(tauri::generate_handler![start_host_session]) // Register command
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}