use crate::transcription::whisper;

/// Configuration for the transcription service
#[derive(Clone)]
pub struct TranscriptionConfig {
    /// Language to use for transcription
    pub language: String,
    /// Minimum duration in seconds required for transcription
    pub min_duration_seconds: f32,
    /// Sample rate expected by the transcription model
    pub sample_rate: usize,
    /// Path to the Whisper model file
    pub model_path: String,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            min_duration_seconds: 1.0,
            sample_rate: 16000,
            model_path: "model/ggml-tiny.en.bin".to_string(),
        }
    }
}

/// TranscriptionService provides an interface to the Whisper transcription.
pub struct TranscriptionService {
    /// Configuration for the transcription service
    config: TranscriptionConfig,
}

impl TranscriptionService {
    /// Creates a new TranscriptionService with default configuration.
    pub fn new() -> Self {
        Self::with_config(TranscriptionConfig::default())
    }

    /// Creates a new TranscriptionService with the specified configuration.
    pub fn with_config(config: TranscriptionConfig) -> Self {
        Self {
            config,
        }
    }

    /// Initializes the transcription service and underlying model.
    ///
    /// # Arguments
    ///
    /// * `model_path` - Path to the Whisper model file
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if successful, Err with error message otherwise
    pub fn initialize(&self) -> Result<(), String> {
        // Initialize the underlying whisper model
        whisper::init(&self.config.model_path)
    }

    /// Transcribes the provided audio samples using Whisper.
    ///
    /// # Arguments
    ///
    /// * `samples` - Audio samples to transcribe
    ///
    /// # Returns
    ///
    /// * `Option<String>` - Transcribed text if successful, None otherwise
    pub fn transcribe(&self, samples: &[f32]) -> Option<String> {
        // Check if we have enough audio
        let min_samples = (self.config.min_duration_seconds * self.config.sample_rate as f32) as usize;
        if samples.len() < min_samples {
            println!("Less than {}s of audio. Skipping...", self.config.min_duration_seconds);
            return None;
        }

        // Use the whisper module to transcribe
        whisper::transcribe(samples)
    }
} 