#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::panic;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use tauri::{AppHandle, Emitter};

mod audio;

#[tauri::command]
fn record_audio(app: AppHandle) {
    let (sender_channel, receiver_channel) = mpsc::channel::<String>();
    tauri::async_runtime::spawn(audio::record_audio(sender_channel));
    tauri::async_runtime::spawn(send_transcrib_chunks_back(app, receiver_channel));
}

async fn send_transcrib_chunks_back(app: AppHandle, receiver_channel: mpsc::Receiver<String>) {
    while let Ok(data) = receiver_channel.recv() {
        app.emit("transcribe", data).unwrap();
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}

fn main() {
    // Install a custom panic hook that logs panic info and delays process termination.
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("A panic occurred: {}", panic_info);
        // Delay termination to allow inspection (e.g., 10 seconds).
        thread::sleep(Duration::from_secs(10));
    }));

    // Optionally, wrap your main application logic in catch_unwind to intercept panics.
    let result = panic::catch_unwind(|| {
      tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, record_audio])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    });

    // If a panic was caught, log and keep the window open.
    if result.is_err() {
        eprintln!("Application encountered an error. Keeping the window open for debugging.");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}

