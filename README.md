# Audio Control

A macOS desktop application for real-time speech-to-text transcription using local processing. Built with Svelte frontend and Tauri (Rust) backend, this application provides minimal latency transcription using the Whisper.rs engine.

## Features

- Real-time speech-to-text transcription
- Local processing - no data leaves your computer
- Minimal latency
- macOS native desktop application
- Built with Svelte and Tauri for optimal performance

## Prerequisites

- macOS
- Node.js (v16 or later)
- Rust (latest stable version)
- Whisper model files (see below)

## Whisper Models

This application uses Whisper.rs for local speech recognition. You'll need to download the appropriate model files:

1. Visit [Whisper.rs GitHub](https://github.com/tazz4843/whisper-rs) for model information
2. Download the model files from [here](https://huggingface.co/ggerganov/whisper.cpp/tree/main) (recommended: `ggml-base.en.bin` for English)
3. Place the model file in the application's model directory in `src-tauri/model`

## Development

To run the application in development mode:

```bash
# Install dependencies
npm install

# Start the development server
npm run tauri dev
```

## Building

To create a production build:

```bash
# Build the application
npm run tauri build
```

The built application will be available in the `src-tauri/target/release` directory.

## Usage

1. Launch the application
2. Select your audio input device
3. Start speaking - your words will be transcribed in real-time
4. The transcribed text will appear in the application window

## Notes

- For best performance, use a model size appropriate for your hardware
- The application processes all audio locally - no internet connection required
- Initial transcription might take a moment as the model loads into memory

## License

[Your License Here]
