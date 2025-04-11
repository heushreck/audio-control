use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration for audio recording parameters
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioRecordingConfig {
    /// Path where recorded audio will be saved
    pub output_path: String,
    /// Whether to save the recorded audio to a file
    pub save_to_file: bool,
    /// Sample rate for the output WAV file (Hz)
    pub output_sample_rate: u32,
    /// Bits per sample for the output WAV file
    pub output_bits_per_sample: u16,
    /// Number of channels for the output WAV file (1 = mono, 2 = stereo)
    pub output_channels: u16,
}

/// Configuration for audio transcription
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioTranscriptionConfig {
    /// Sample rate required by the Whisper model (Hz)
    pub whisper_sample_rate: u32,
    /// Minimum number of samples needed for transcription
    pub min_transcription_samples: usize,
    /// Language to use for transcription
    pub language: String,
    /// Minimum duration in seconds required for transcription
    pub min_duration_seconds: f32,
    /// Path to the Whisper model file
    pub path_to_model: String,
}

/// Configuration for audio processing performance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioPerformanceConfig {
    /// Buffer capacity for async channels
    pub channel_buffer_size: usize,
}

/// Combined audio configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioConfig {
    /// Recording-specific configuration
    pub recording: AudioRecordingConfig,
    /// Transcription-specific configuration
    pub transcription: AudioTranscriptionConfig,
    /// Performance-related configuration
    pub performance: AudioPerformanceConfig,
}

/// Command detection configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandConfig {
    /// Trigger word that activates command detection
    pub trigger_word: String,
    /// Minimum confidence required to recognize a command
    pub min_confidence: f32,
}

/// Top-level application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    /// Audio processing configuration
    pub audio: AudioConfig,
    /// Command detection configuration
    #[serde(default)]
    pub commands: CommandConfig,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            trigger_word: "hey computer".to_string(),
            min_confidence: 0.7,
        }
    }
}

impl AppConfig {
    /// Loads configuration from the specified YAML file.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn std::error::Error>>` - The loaded configuration or an error
    pub fn load(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Check if the file exists
        if !Path::new(config_path).exists() {
            return Err(format!("Config file not found at {}", config_path).into());
        }
        
        // Read and parse the file
        let config_content = fs::read_to_string(config_path)?;
        let config: AppConfig = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
    
    /// Creates a default configuration with reasonable values.
    ///
    /// # Returns
    ///
    /// * `Self` - A default configuration
    pub fn default() -> Self {
        Self {
            audio: AudioConfig {
                recording: AudioRecordingConfig {
                    output_path: "output.wav".to_string(),
                    save_to_file: true,
                    output_sample_rate: 44100,
                    output_bits_per_sample: 16,
                    output_channels: 1,
                },
                transcription: AudioTranscriptionConfig {
                    whisper_sample_rate: 16000,
                    min_transcription_samples: 49000,
                    language: "en".to_string(),
                    min_duration_seconds: 1.0,
                    path_to_model: "model/ggml-tiny.en.bin".to_string(),
                },
                performance: AudioPerformanceConfig {
                    channel_buffer_size: 16,
                },
            },
            commands: CommandConfig::default(),
        }
    }
} 