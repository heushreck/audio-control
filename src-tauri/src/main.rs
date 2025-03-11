#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{sync::Mutex};
use std::sync::Arc;
use std::sync::mpsc;
use tauri::{AppHandle, Emitter};

mod whisper;
mod audio;
use audio::Recorder;


#[tauri::command]
fn start_recording(app: AppHandle, recorder: tauri::State<Arc<Mutex<Recorder>>>) {
    // Clone the Arc from the state so it can be moved into the async block.
    let recorder_arc = recorder.inner().clone();
    let (sender_channel, receiver_channel) = mpsc::channel::<String>();

    // Spawn an async task that locks the recorder and starts recording.
    tauri::async_runtime::spawn(async move {
        let mut rec = recorder_arc.lock().unwrap();
        rec.start_recording(sender_channel);
    });

    // Spawn the async task that sends transcription chunks back.
    tauri::async_runtime::spawn(send_transcrib_chunks_back(app, receiver_channel));
}


#[tauri::command]
fn stop_recording(recorder: tauri::State<Arc<Mutex<Recorder>>>) {
    let mut recorder = recorder.lock().unwrap();
    recorder.stop_recording();
}

async fn send_transcrib_chunks_back(app: AppHandle, receiver_channel: mpsc::Receiver<String>) {
    while let Ok(data) = receiver_channel.recv() {
        app.emit("transcribe", data).unwrap();
    }
}

fn main() {
    // Create the recorder instance
    let recorder = Arc::new(Mutex::new(Recorder::new()));

    let path_to_model = "model/ggml-tiny.en.bin";

    whisper::init(&path_to_model);

    tauri::Builder::default()
        .manage(recorder.clone()) // Share the recorder state with Tauri commands
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

}

