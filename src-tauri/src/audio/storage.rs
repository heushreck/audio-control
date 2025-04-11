use std::sync::{Arc, Mutex};
use hound;

/// Configuration for audio storage
#[derive(Clone)]
pub struct StorageConfig {
    /// Path where audio will be saved
    pub output_path: String,
    /// Whether to save audio to file
    pub save_to_file: bool,
    /// Sample rate for the output WAV file
    pub output_sample_rate: u32,
    /// Number of channels for the output WAV file
    pub output_channels: u16,
    /// Bits per sample for the output WAV file
    pub output_bits_per_sample: u16,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            output_path: "output.wav".to_string(),
            save_to_file: true,
            output_sample_rate: 44100,
            output_channels: 1,
            output_bits_per_sample: 16,
        }
    }
}

/// AudioStorage handles saving audio to files.
pub struct AudioStorage {
    /// Buffer storing all recorded samples
    recorded_samples: Arc<Mutex<Vec<f32>>>,
    /// Configuration for storage
    config: StorageConfig,
}

impl AudioStorage {
    /// Creates a new AudioStorage with default configuration.
    pub fn new() -> Self {
        Self::with_config(StorageConfig::default())
    }

    /// Creates a new AudioStorage with the specified configuration.
    pub fn with_config(config: StorageConfig) -> Self {
        Self {
            recorded_samples: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Adds samples to the storage buffer.
    ///
    /// # Arguments
    ///
    /// * `samples` - Audio samples to add
    pub fn add_samples(&self, samples: &[f32]) {
        let mut buffer = self.recorded_samples.lock().unwrap();
        buffer.extend_from_slice(samples);
    }

    /// Saves the recorded audio to a WAV file.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Ok if successful, Err with error message otherwise
    pub fn save(&self) -> Result<(), String> {
        // Check if saving is enabled
        if !self.config.save_to_file {
            return Ok(());
        }

        // Create WAV spec
        let spec = hound::WavSpec {
            channels: self.config.output_channels,
            sample_rate: self.config.output_sample_rate,
            bits_per_sample: self.config.output_bits_per_sample,
            sample_format: hound::SampleFormat::Int,
        };

        // Create WAV writer
        let mut writer = match hound::WavWriter::create(&self.config.output_path, spec) {
            Ok(writer) => writer,
            Err(err) => return Err(format!("Failed to create WAV writer: {}", err)),
        };

        // Get samples and write to file
        let samples = self.recorded_samples.lock().unwrap();
        for &sample in samples.iter() {
            let clamped = (sample * i16::MAX as f32)
                .max(i16::MIN as f32)
                .min(i16::MAX as f32) as i16;
            if let Err(err) = writer.write_sample(clamped) {
                return Err(format!("Failed to write sample: {}", err));
            }
        }

        // Finalize the file
        if let Err(err) = writer.finalize() {
            return Err(format!("Failed to finalize WAV file: {}", err));
        }

        println!("WAV file written to {}", self.config.output_path);
        Ok(())
    }

    /// Clears the recorded samples buffer.
    pub fn clear(&self) {
        let mut buffer = self.recorded_samples.lock().unwrap();
        buffer.clear();
    }
} 