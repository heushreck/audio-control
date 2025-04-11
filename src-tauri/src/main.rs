#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc;
use tauri::{AppHandle, Emitter};

mod audio;
mod config;
mod transcription;
mod orchestrator;
mod command;

use config::AppConfig;
use orchestrator::Orchestrator;
use audio::recorder::Recorder;
use audio::processor::AudioProcessor;
use audio::storage::AudioStorage;
use transcription::service::TranscriptionService;
use command::detector::CommandDetector;

// Import the specific configuration structs
use audio::recorder::RecorderConfig;
use audio::processor::ProcessorConfig;
use audio::storage::StorageConfig;
use transcription::service::TranscriptionConfig;
use command::detector::CommandDetectorConfig;

#[tauri::command]
fn start_recording(app: AppHandle, orchestrator: tauri::State<Arc<Mutex<Orchestrator>>>) {
    // Clone the Arc from the state
    let orchestrator_arc = orchestrator.inner().clone();
    let (sender_channel, receiver_channel) = mpsc::channel::<String>();

    // Spawn an async task that starts the orchestrator
    tauri::async_runtime::spawn(async move {
        let mut orchestrator = orchestrator_arc.lock().unwrap();
        orchestrator.start(sender_channel);
    });

    // Spawn the async task that sends transcription chunks back
    tauri::async_runtime::spawn(send_transcribe_chunks_back(app, receiver_channel));
}

#[tauri::command]
fn stop_recording(orchestrator: tauri::State<Arc<Mutex<Orchestrator>>>) {
    // Clone the Arc
    let orchestrator_arc = orchestrator.inner().clone();
    
    // Use tokio's async runtime
    tauri::async_runtime::spawn(async move {
        let mut orchestrator = orchestrator_arc.lock().unwrap();
        orchestrator.stop();
    });
}

async fn send_transcribe_chunks_back(app: AppHandle, receiver_channel: mpsc::Receiver<String>) {
    while let Ok(data) = receiver_channel.recv() {
        if let Err(err) = app.emit("transcribe", data) {
            eprintln!("Failed to emit transcription event: {:?}", err);
        }
    }
}

fn main() {
    // Define configuration file paths
    let config_path = "../config.yaml";
    
    // Load configuration from file or use defaults
    let app_config = AppConfig::load(config_path).unwrap_or_else(|e| {
        eprintln!("Configuration error: {}. Using defaults.", e);
        AppConfig::default()
    });

    // Create configurations for components
    let recorder_config = RecorderConfig {
        channels: app_config.audio.recording.output_channels,
        sample_rate: app_config.audio.recording.output_sample_rate,
    };

    let processor_config = ProcessorConfig {
        target_sample_rate: app_config.audio.transcription.whisper_sample_rate,
        target_channels: 1, // Whisper expects mono
        source_sample_rate: app_config.audio.recording.output_sample_rate,
        source_channels: app_config.audio.recording.output_channels,
        min_samples_for_processing: app_config.audio.transcription.min_transcription_samples,
        max_buffer_size: app_config.audio.transcription.min_transcription_samples * 10, // 10 times the min size
    };

    let storage_config = StorageConfig {
        output_path: app_config.audio.recording.output_path.clone(),
        save_to_file: app_config.audio.recording.save_to_file,
        output_sample_rate: app_config.audio.recording.output_sample_rate,
        output_channels: app_config.audio.recording.output_channels,
        output_bits_per_sample: app_config.audio.recording.output_bits_per_sample,
    };

    let transcription_config = TranscriptionConfig {
        language: app_config.audio.transcription.language.clone(),
        min_duration_seconds: app_config.audio.transcription.min_duration_seconds,
        sample_rate: app_config.audio.transcription.whisper_sample_rate as usize,
        model_path: app_config.audio.transcription.path_to_model.clone(),
    };

    let command_detector_config = CommandDetectorConfig::default();

    // Create component instances
    let recorder = Recorder::with_config(recorder_config);
    let processor = AudioProcessor::with_config(processor_config);
    let _storage = AudioStorage::with_config(storage_config);
    let transcription_service = TranscriptionService::with_config(transcription_config);
    
    // Initialize the transcription service
    if let Err(e) = transcription_service.initialize() {
        eprintln!("Failed to initialize transcription service: {}", e);
        std::process::exit(1);
    }
    
    let _command_detector = CommandDetector::with_config(command_detector_config);

    // Create the orchestrator
    let orchestrator = Orchestrator::new(
        recorder,
        processor,
        transcription_service,
        app_config.clone(),
    );
    let orchestrator = Arc::new(Mutex::new(orchestrator));

    tauri::Builder::default()
        .manage(orchestrator.clone()) // Share the orchestrator state
        .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

