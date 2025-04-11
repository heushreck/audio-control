use std::sync::{Arc, Mutex, mpsc};
use tokio::sync::mpsc as tokio_mpsc;
use tokio::task;

use crate::audio::recorder::Recorder;
use crate::audio::processor::AudioProcessor;
use crate::transcription::service::TranscriptionService;
use crate::config::AppConfig;

/// Orchestrator manages the high-level flow of the application.
/// It coordinates between audio recording, processing, and transcription.
pub struct Orchestrator {
    /// Audio recorder component
    recorder: Arc<Mutex<Recorder>>,
    /// Audio processor component
    processor: Arc<Mutex<AudioProcessor>>,
    /// Transcription service component
    transcription_service: Arc<Mutex<TranscriptionService>>,
    /// Global app configuration
    app_config: Arc<Mutex<AppConfig>>,
    /// Handle to the orchestration task
    orchestration_handle: Option<task::JoinHandle<()>>,
    /// Flag indicating whether the orchestrator is active
    is_active: Arc<Mutex<bool>>,
    /// Signal to stop the orchestration
    stop_signal: Arc<Mutex<bool>>,
}

impl Orchestrator {
    /// Creates a new Orchestrator with the provided components and configuration.
    pub fn new(
        recorder: Recorder,
        processor: AudioProcessor,
        transcription_service: TranscriptionService,
        app_config: AppConfig,
    ) -> Self {
        Self {
            recorder: Arc::new(Mutex::new(recorder)),
            processor: Arc::new(Mutex::new(processor)),
            transcription_service: Arc::new(Mutex::new(transcription_service)),
            app_config: Arc::new(Mutex::new(app_config)),
            orchestration_handle: None,
            is_active: Arc::new(Mutex::new(false)),
            stop_signal: Arc::new(Mutex::new(false)),
        }
    }

    /// Starts the orchestration process, recording audio and transcribing it.
    pub fn start(&mut self, transcribe_channel: mpsc::Sender<String>) {
        // Check if already active
        {
            let active = self.is_active.lock().unwrap();
            if *active {
                println!("Orchestrator already active");
                return;
            }
        }

        // Mark as active
        {
            let mut active = self.is_active.lock().unwrap();
            *active = true;
        }

        // Reset stop signal
        {
            let mut stop_signal = self.stop_signal.lock().unwrap();
            *stop_signal = false;
        }

        // Get configuration values
        let app_config = self.app_config.clone();
        let app_config_guard = app_config.lock().unwrap();
        let channel_buffer_size = app_config_guard.audio.performance.channel_buffer_size;
        
        // Create a channel for audio data between recorder and processor
        let (audio_sender, audio_receiver) = tokio_mpsc::channel(channel_buffer_size);

        // Start the recorder
        {
            let mut recorder = self.recorder.lock().unwrap();
            recorder.start_recording(audio_sender);
        }

        // Clone needed values for the async task
        let processor = self.processor.clone();
        let transcription_service = self.transcription_service.clone();
        let stop_signal = self.stop_signal.clone();
        let transcribe_channel_clone = transcribe_channel.clone();

        // Start the orchestration task
        let handle = tokio::spawn(async move {
            // Process audio chunks and transcribe them
            let mut audio_receiver = audio_receiver;
            let mut consecutive_failures = 0;
            
            while !*stop_signal.lock().unwrap() {
                // Process audio chunks from recorder
                match audio_receiver.recv().await {
                    Some(chunk) => {
                        // Process the audio chunk
                        match processor.lock().unwrap().process(chunk) {
                            Some(processed_audio) => {
                                // Reset failure counter on success
                                consecutive_failures = 0;
                                
                                // Transcribe the processed audio
                                match transcription_service.lock().unwrap().transcribe(&processed_audio) {
                                    Some(text) => {
                                        // Send transcription result back
                                        if let Err(err) = transcribe_channel_clone.send(text) {
                                            println!("Failed to send transcription: {}", err);
                                        }
                                    },
                                    None => {
                                        // Transcription returned none (not enough audio, etc.)
                                        // This is expected in some cases, no action needed
                                    }
                                }
                            },
                            None => {
                                // Processing returned none (not enough audio, etc.)
                                // This is expected in some cases, no action needed
                            }
                        }
                    },
                    None => {
                        // Channel closed or error receiving
                        consecutive_failures += 1;
                        if consecutive_failures > 5 {
                            println!("Too many failures receiving audio, stopping orchestration");
                            break;
                        }
                        // Sleep briefly to avoid tight loop
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                }
            }
            println!("Orchestration task stopped");
        });

        self.orchestration_handle = Some(handle);
    }

    /// Stops the orchestration process.
    pub fn stop(&mut self) {
        // Check if active
        {
            let active = self.is_active.lock().unwrap();
            if !*active {
                println!("Orchestrator not active");
                return;
            }
        }

        // Signal tasks to stop
        {
            let mut stop_signal = self.stop_signal.lock().unwrap();
            *stop_signal = true;
        }

        // Mark as inactive
        {
            let mut active = self.is_active.lock().unwrap();
            *active = false;
        }

        // Stop the recorder
        {
            let mut recorder = self.recorder.lock().unwrap();
            recorder.stop_recording();
        }

        println!("Orchestration stopped");
    }
} 