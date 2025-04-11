use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc as tokio_mpsc;
use std::thread;

/// Configuration for the audio recorder
#[derive(Clone)]
pub struct RecorderConfig {
    /// Number of channels for recording (1=mono, 2=stereo)
    pub channels: u16,
    /// Sample rate for recording
    pub sample_rate: u32,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            channels: 1,
            sample_rate: 44100,
        }
    }
}

/// State of the recording process
#[derive(Clone, Copy, PartialEq)]
pub enum RecordingState {
    /// Recording is inactive
    Inactive,
    /// Recording is active
    Active,
    /// Recording should stop
    StopRequested,
}

/// Thread-safe recorder state that can be shared between threads
struct RecorderState {
    /// Current state of recording
    recording_state: RecordingState,
    /// Channel to send audio data
    audio_sender: Option<tokio_mpsc::Sender<Vec<f32>>>,
}

/// Recorder handles capturing audio from the microphone.
/// This implementation uses a thread-local pattern for CPAL to make it thread-safe.
pub struct Recorder {
    /// Shared state between threads
    state: Arc<Mutex<RecorderState>>,
    /// Configuration for the recorder
    config: RecorderConfig,
    /// Handle to the recording thread
    recording_thread: Option<thread::JoinHandle<()>>,
}

// Safe to send between threads because we've isolated the non-Send types
unsafe impl Send for Recorder {}

impl Recorder {
    /// Creates a new Recorder with default configuration.
    pub fn new() -> Self {
        Self::with_config(RecorderConfig::default())
    }

    /// Creates a new Recorder with the specified configuration.
    pub fn with_config(config: RecorderConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(RecorderState {
                recording_state: RecordingState::Inactive,
                audio_sender: None,
            })),
            config,
            recording_thread: None,
        }
    }

    /// Starts recording audio from the default input device.
    ///
    /// This method launches a dedicated thread for audio recording and sends audio chunks
    /// to the provided channel.
    ///
    /// # Arguments
    ///
    /// * `audio_sender` - Channel to send recorded audio chunks
    pub fn start_recording(&mut self, audio_sender: tokio_mpsc::Sender<Vec<f32>>) {
        // Check if already recording
        {
            let state = self.state.lock().unwrap();
            if state.recording_state == RecordingState::Active {
                println!("Recording is already active");
                return;
            }
        }
        
        // Set up the state for recording
        {
            let mut state = self.state.lock().unwrap();
            state.recording_state = RecordingState::Active;
            state.audio_sender = Some(audio_sender);
        }
        
        // Clone what we need for the thread
        let state = self.state.clone();
        let config = self.config.clone();
        
        // Launch a dedicated thread for audio recording
        let recording_thread = thread::spawn(move || {
            Self::record_audio_thread(state, config);
        });
        
        self.recording_thread = Some(recording_thread);
        println!("Recording started...");
    }

    /// Dedicated thread function for audio recording.
    /// 
    /// This runs in its own thread to isolate the CPAL non-Send types.
    ///
    /// # Arguments
    ///
    /// * `state` - Shared recorder state
    /// * `config` - Recorder configuration
    fn record_audio_thread(state: Arc<Mutex<RecorderState>>, _config: RecorderConfig) {
        // Get the host and device
        let host = cpal::default_host();
        let device = match host.default_input_device() {
            Some(device) => device,
            None => {
                println!("No input device available");
                return;
            }
        };

        // Get the default input config
        let device_config = match device.default_input_config() {
            Ok(config) => config,
            Err(err) => {
                println!("Failed to get default input config: {}", err);
                return;
            }
        };

        println!("Default input config: {:?}", device_config);
        let sample_format = device_config.sample_format();
        let channels = device_config.channels();
        let sample_rate = device_config.sample_rate().0;
        println!("Sample format: {:?} Channels: {} Sample rate: {}", sample_format, channels, sample_rate);
        
        let cpal_config: cpal::StreamConfig = device_config.into();
        
        // Get the sender from shared state
        let audio_sender = {
            let state = state.lock().unwrap();
            match &state.audio_sender {
                Some(sender) => sender.clone(),
                None => {
                    println!("No audio sender available");
                    return;
                }
            }
        };
        
        // Function to check if recording should stop
        let should_stop = Arc::new(move || {
            let state = state.lock().unwrap();
            state.recording_state != RecordingState::Active
        });
        
        // Clone for the error callback
        let should_stop_err = should_stop.clone();

        // Build the stream
        let err_fn = move |err| {
            eprintln!("Error on stream: {}", err);
            if should_stop_err() {
                println!("Recording stopped due to error");
            }
        };

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                // Clone for the data callback
                let should_stop_data = should_stop.clone();
                let data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Check if we should stop
                    if should_stop_data() {
                        return;
                    }
                    
                    // Clone the data and send it to the processor
                    let data_vec = data.to_vec();
                    if let Err(err) = audio_sender.try_send(data_vec) {
                        match err {
                            tokio_mpsc::error::TrySendError::Full(_) => {
                                // Channel is full, which means processing is slow
                                println!("Audio processing is falling behind - channel full");
                            },
                            tokio_mpsc::error::TrySendError::Closed(_) => {
                                // Channel is closed, which means processing has stopped
                                println!("Audio channel closed");
                            }
                        }
                    }
                };
                
                device.build_input_stream(&cpal_config, data_fn, err_fn, None)
            },
            _ => {
                println!("Unsupported sample format: {:?}", sample_format);
                return;
            }
        };

        // Check if stream was created successfully
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                println!("Failed to build input stream: {}", err);
                return;
            }
        };

        // Start the stream
        if let Err(err) = stream.play() {
            println!("Failed to start stream: {}", err);
            return;
        }

        // Keep the thread alive until recording should stop
        while !should_stop() {
            // Sleep to avoid busy waiting
            thread::sleep(std::time::Duration::from_millis(100));
        }
        
        // Stream will be dropped when this thread ends
        println!("Recording thread stopped");
    }

    /// Stops the active recording session.
    pub fn stop_recording(&mut self) {
        // Signal the recording thread to stop
        {
            let mut state = self.state.lock().unwrap();
            if state.recording_state != RecordingState::Active {
                println!("Recording is not active.");
                return;
            }
            state.recording_state = RecordingState::StopRequested;
        }
        
        println!("Stopping recording...");
        
        // Wait for the recording thread to finish
        if let Some(thread) = self.recording_thread.take() {
            // Don't wait indefinitely - use a timeout
            let _ = thread.join();
        }
        
        // Reset the state
        {
            let mut state = self.state.lock().unwrap();
            state.recording_state = RecordingState::Inactive;
            state.audio_sender = None;
        }
        
        println!("Recording stopped");
    }
} 