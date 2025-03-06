use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use hound;

pub struct Recorder {
    is_recording: Arc<Mutex<bool>>,
    recorded_samples: Arc<Mutex<Vec<f32>>>,
    stop_signal: Arc<Mutex<bool>>,
    microphone_stream: Option<cpal::Stream>,
    transcribe_thread: Option<std::thread::JoinHandle<()>>,
}

unsafe impl Send for Recorder {}
unsafe impl Sync for Recorder {}

impl Recorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
            recorded_samples: Arc::new(Mutex::new(Vec::new())),
            stop_signal: Arc::new(Mutex::new(false)),
            microphone_stream: None,
            transcribe_thread: None,
        }
    }

    pub fn start_recording(&mut self, transcribe_channel: mpsc::Sender<String>) {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device available");
        let config = device.default_input_config().expect("Failed to get default input config");
        let sample_format = config.sample_format();
        let config: cpal::StreamConfig = config.into();

        let recorded_samples_clone = self.recorded_samples.clone();
        // let is_recording_clone = self.is_recording.clone();

        let (sender_channel, receiver_channel) = mpsc::channel::<Vec<f32>>();

        // Set recording state to true
        {
            let mut recording = self.is_recording.lock().unwrap();
            *recording = true;
        }

        

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    let mut samples = recorded_samples_clone.lock().unwrap();
                    samples.extend_from_slice(data);
                    let _ = sender_channel.send(data.to_vec());
                },
                move |err| eprintln!("Error on stream: {}", err),
                None,
            ),
            _ => panic!("Unsupported sample format"),
        }
        .expect("Failed to build input stream");

        stream.play().expect("Failed to start stream");

        self.microphone_stream = Some(stream);

        {
            let mut stop_signal = self.stop_signal.lock().unwrap();
            *stop_signal = true;
        }

        let stop_signal_clone = self.stop_signal.clone();

        println!("Recording started...");
        // Spawn a transcription thread
        let transcribe_handle = thread::spawn(move || {
            let mut i = 0;
            while *stop_signal_clone.lock().unwrap() {
                if let Ok(data) = receiver_channel.recv() {
                    if i % 50 == 0 {
                        println!("Received audio chunk #{} with {} samples", i, data.len());
                        let _ = transcribe_channel.send("Nicolas ".to_string());
                    }
                    i += 1;
                }
            }
            println!("Transcription thread stopped.");
        });

        self.transcribe_thread = Some(transcribe_handle);
    }

    pub fn stop_recording(&mut self) {

        {
            let mut recording = self.is_recording.lock().unwrap();
            if !*recording {
                println!("Recording is not active.");
                return;
            }
        }

        {
            let mut stop_signal = self.stop_signal.lock().unwrap();
            *stop_signal = false;
        }

        {
            let mut recording = self.is_recording.lock().unwrap();
            *recording = false;
        }

        println!("Stopping recording...");

        if let Some(thread_handle) = self.transcribe_thread.take() {
            thread_handle.join().expect("Failed to join transcription thread");
        }
        // drop the microphone stream
        drop(self.microphone_stream.take());

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let path = "output.wav";
        let mut writer = hound::WavWriter::create(path, spec).expect("Failed to create WAV writer");

        let samples = self.recorded_samples.lock().unwrap();
        for &sample in samples.iter() {
            let clamped = (sample * i16::MAX as f32)
                .max(i16::MIN as f32)
                .min(i16::MAX as f32) as i16;
            writer.write_sample(clamped).expect("Failed to write sample");
        }
        writer.finalize().expect("Failed to finalize WAV file");
        println!("WAV file written to {}", path);
    }
}
