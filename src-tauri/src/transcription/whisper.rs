use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

/// Configuration constants
const MIN_AUDIO_DURATION_SECONDS: f32 = 1.0;
const SAMPLE_RATE: usize = 16_000;
const DEFAULT_LANGUAGE: &str = "en";

/// Global Whisper state shared across the application
static WHISPER_STATE: Lazy<Arc<Mutex<Option<WhisperState>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Global Whisper parameters
static WHISPER_PARAMS: Lazy<Mutex<Option<FullParams>>> = Lazy::new(|| Mutex::new(None));

/// Initializes the Whisper speech-to-text model with the specified model file.
///
/// This function loads the model from the provided path and configures it with
/// default parameters optimized for English transcription.
///
/// # Arguments
///
/// * `model_path` - Path to the Whisper model file
///
/// # Returns
///
/// * `Result<(), String>` - Ok if successful, Err with error message otherwise
pub fn init(model_path: &str) -> Result<(), String> {
    // Create Whisper context
    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| format!("Failed to create Whisper context: {:?}", e))?;
    
    // Create state
    let state = ctx.create_state()
        .map_err(|e| format!("Failed to create Whisper state: {:?}", e))?;
    
    whisper_rs::install_whisper_tracing_trampoline();
    
    // Configure parameters
    let mut params = FullParams::new(SamplingStrategy::default());
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    params.set_language(Some(DEFAULT_LANGUAGE));

    // Store state and parameters
    {
        let mut global_state = WHISPER_STATE.lock()
            .map_err(|_| "Failed to lock Whisper state".to_string())?;
        *global_state = Some(state);
    }
    
    {
        let mut global_params = WHISPER_PARAMS.lock()
            .map_err(|_| "Failed to lock Whisper parameters".to_string())?;
        *global_params = Some(params);
    }

    Ok(())
}

/// Transcribes the provided audio samples using the Whisper model.
///
/// This function uses the globally initialized Whisper state to transcribe
/// the provided audio samples. The audio must be in 16kHz sampling rate format.
/// Returns None if the audio is too short (less than 1 second).
///
/// # Arguments
///
/// * `samples` - Audio samples as f32 values (16kHz, mono)
///
/// # Returns
///
/// * `Option<String>` - Transcribed text if successful, None otherwise
pub fn transcribe(samples: &[f32]) -> Option<String> {
    let min_samples = (MIN_AUDIO_DURATION_SECONDS * SAMPLE_RATE as f32) as usize;
    if samples.len() < min_samples {
        println!("Less than {}s of audio. Skipping...", MIN_AUDIO_DURATION_SECONDS);
        return None;
    }

    // Get state and parameters
    let state_lock = WHISPER_STATE.clone();
    let mut state_guard = match state_lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            println!("Failed to lock Whisper state");
            return None;
        }
    };
    
    let state = match state_guard.as_mut() {
        Some(state) => state,
        None => {
            println!("Whisper state not initialized");
            return None;
        }
    };
    
    let params_guard = match WHISPER_PARAMS.lock() {
        Ok(guard) => guard,
        Err(_) => {
            println!("Failed to lock Whisper parameters");
            return None;
        }
    };
    
    let mut params = match params_guard.clone() {
        Some(params) => params,
        None => {
            println!("Whisper parameters not initialized");
            return None;
        }
    };

    // Configure parameters for this run
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    params.set_language(Some(DEFAULT_LANGUAGE));

    println!("Transcribing...");

    // Process the audio
    if let Err(err) = state.full(params, samples) {
        println!("Failed to transcribe audio: {:?}", err);
        return None;
    }

    println!("Got State");
    
    // Get the transcription result
    match state.full_get_segment_text_lossy(0) {
        Ok(text) => {
            println!("Returned text");
            Some(text)
        },
        Err(err) => {
            println!("Failed to get segment text: {:?}", err);
            None
        }
    }
} 