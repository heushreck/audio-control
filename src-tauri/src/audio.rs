use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::mpsc;
use std::thread;
use hound;

pub async fn record_audio(transcribe_channel: mpsc::Sender<String>) {
    // Get the default audio host and input device.
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = device.default_input_config().expect("Failed to get default input config");
    let sample_format = config.sample_format();
    let config: cpal::StreamConfig = config.into();

    // Shared buffer to store recorded samples.
    let recorded_samples = Arc::new(Mutex::new(Vec::<f32>::new()));

    // Channel to send audio chunks for transcription.
    let (sender_channel, receiver_channel) = mpsc::channel::<Vec<f32>>();

    // Spawn a thread to process audio chunks.
    let transcribe_thread = thread::spawn(move || {
        let mut i = 0;
        while let Ok(data) = receiver_channel.recv() {
            // Process audio data here.
            if i % 10 == 0 {
                println!("Received audio chunk #{} with {} samples", i, data.len());
                let _ = transcribe_channel.send("Nicolas ".to_string());
            }
            i += 1;
        }
    });

    // Build the input stream based on the sample format.
    let recorded_samples_clone = recorded_samples.clone();
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
    }.expect("Failed to build input stream");

    println!("Recording for 5 seconds...");
    // Start recording.
    stream.play().expect("Failed to start stream");
    thread::sleep(Duration::from_secs(5));
    drop(stream); // Stops recording after 5 seconds.
    transcribe_thread.join().expect("Failed to join transcribe thread");
    println!("Recording stopped");

    // Set up WAV writer specifications.
    let spec = hound::WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Write the recorded samples to "output.wav".
    let path = "output.wav";
    let mut writer = hound::WavWriter::create(path, spec)
        .expect("Failed to create WAV writer");

    let samples = recorded_samples.lock().unwrap();
    for &sample in samples.iter() {
        let clamped = (sample * i16::MAX as f32)
            .max(i16::MIN as f32)
            .min(i16::MAX as f32) as i16;
        writer.write_sample(clamped).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize WAV file");
    println!("WAV file written to {}", path);
}
