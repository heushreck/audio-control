use std::sync::{Arc, Mutex};

/// Configuration for the audio processor
#[derive(Clone)]
pub struct ProcessorConfig {
    /// Sample rate for processing (the rate Whisper expects)
    pub target_sample_rate: u32,
    /// Channels for processing (typically mono for Whisper)
    pub target_channels: u16,
    /// Source sample rate (from the recorder)
    pub source_sample_rate: u32,
    /// Source channels (from the recorder)
    pub source_channels: u16,
    /// Minimum number of samples needed for processing
    pub min_samples_for_processing: usize,
    /// Maximum buffer size to prevent memory issues
    pub max_buffer_size: usize,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            target_sample_rate: 16000, // Whisper typically expects 16kHz
            target_channels: 1,        // Whisper expects mono
            source_sample_rate: 44100, // Typical recording sample rate
            source_channels: 1,        // Recorder is typically set to mono
            min_samples_for_processing: 16000, // At least 1 second of audio at 16kHz
            max_buffer_size: 160000,   // Prevent excessive memory use (10 seconds at 16kHz)
        }
    }
}

/// AudioProcessor handles audio processing, buffering, and resampling.
pub struct AudioProcessor {
    /// Buffer storing audio samples until enough for processing
    buffer: Arc<Mutex<Vec<f32>>>,
    /// Configuration for the processor
    config: ProcessorConfig,
}

impl AudioProcessor {
    /// Creates a new AudioProcessor with default configuration.
    pub fn new() -> Self {
        Self::with_config(ProcessorConfig::default())
    }

    /// Creates a new AudioProcessor with the specified configuration.
    pub fn with_config(config: ProcessorConfig) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(config.min_samples_for_processing * 2))),
            config,
        }
    }

    /// Processes an audio chunk, buffering until enough samples are available,
    /// then resampling the audio to the target sample rate and channels.
    ///
    /// # Arguments
    ///
    /// * `chunk` - Audio samples to process
    ///
    /// # Returns
    ///
    /// * `Option<Vec<f32>>` - Processed audio ready for transcription, or None if not enough samples
    pub fn process(&self, chunk: Vec<f32>) -> Option<Vec<f32>> {
        // Add the new chunk to the buffer
        let mut buffer_guard = self.buffer.lock().unwrap();
        
        // If buffer is getting too large, clear part of it to prevent memory issues
        if buffer_guard.len() > self.config.max_buffer_size {
            // Keep only the most recent portion
            let start_idx = buffer_guard.len() - self.config.min_samples_for_processing;
            buffer_guard.copy_within(start_idx.., 0);
            buffer_guard.truncate(self.config.min_samples_for_processing);
            println!("Buffer too large, trimmed to recent samples only");
        }
        
        buffer_guard.extend(chunk);
        
        // If we don't have enough samples to process yet, return None
        if buffer_guard.len() < self.config.min_samples_for_processing {
            return None;
        }
        
        // Use all accumulated samples for better speech recognition
        let samples_to_process = buffer_guard.clone();
        
        // Clear the buffer after processing
        buffer_guard.clear();
        
        // Resample if necessary
        if self.config.source_sample_rate != self.config.target_sample_rate ||
           self.config.source_channels != self.config.target_channels {
            let resampled = self.resample(&samples_to_process);
            Some(resampled)
        } else {
            Some(samples_to_process)
        }
    }

    /// Resamples audio from source to target sample rate and channels.
    ///
    /// # Arguments
    ///
    /// * `samples` - Audio samples to resample
    ///
    /// # Returns
    ///
    /// * `Vec<f32>` - Resampled audio
    fn resample(&self, samples: &[f32]) -> Vec<f32> {
        // Use vad_rs for resampling (which is already in the project's dependencies)
        vad_rs::audio_resample(
            samples,
            self.config.source_sample_rate,
            self.config.target_sample_rate,
            self.config.source_channels
        )
    }
} 