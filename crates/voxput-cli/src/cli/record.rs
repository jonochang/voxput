use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Args;
use crossterm::event::{self, Event};
use crossterm::terminal;
use voxput_core::audio::cpal_backend::CpalBackend;
use voxput_core::audio::wav::encode_wav;
use voxput_core::audio::AudioBackend;
use voxput_core::config;
use voxput_core::errors::Result;
use voxput_core::output::{self, OutputTarget};
use voxput_core::provider::groq::GroqProvider;
use voxput_core::provider::{TranscribeOptions, TranscriptionProvider};
use voxput_core::state::{DictationEvent, DictationStateMachine};

#[derive(Debug, Args)]
pub struct RecordArgs {
    /// Max recording duration in seconds (0 = record until any key is pressed)
    #[arg(long, short, default_value = "0")]
    pub duration: f32,

    /// Output target
    #[arg(long, short, default_value = "stdout")]
    pub output: OutputTarget,

    /// Audio input device name (omit to use system default)
    #[arg(long)]
    pub device: Option<String>,

    /// Language hint (ISO 639-1, e.g. "en")
    #[arg(long)]
    pub language: Option<String>,

    /// Transcription model (overrides config)
    #[arg(long)]
    pub model: Option<String>,

    /// Print transcript as JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: &RecordArgs) -> Result<()> {
    let mut sm = DictationStateMachine::new();

    let config = config::load_config()?;
    let api_key = config.api_key()?;
    let model = args.model.clone().or(config.model.clone());

    // Shared flag: set by keypress listener or by the duration timer.
    let stop = Arc::new(AtomicBool::new(false));

    // Spawn keypress listener thread.
    let stop_for_listener = Arc::clone(&stop);
    let _listener = std::thread::spawn(move || {
        if terminal::enable_raw_mode().is_err() {
            return;
        }
        loop {
            // Poll so we can also notice when the recording side has set the flag.
            match event::poll(Duration::from_millis(50)) {
                Ok(true) => {
                    // Consume the event; any key press stops recording.
                    if let Ok(Event::Key(_)) = event::read() {
                        stop_for_listener.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                Ok(false) => {
                    // No event — check if the recording side already stopped us.
                    if stop_for_listener.load(Ordering::Relaxed) {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = terminal::disable_raw_mode();
    });

    // Print recording prompt.
    sm.handle(DictationEvent::StartRecording);
    if args.duration > 0.0 {
        eprintln!("Recording… press any key to stop (max {:.0}s)", args.duration);
    } else {
        eprintln!("Recording… press any key to stop");
    }

    let backend = CpalBackend;
    let audio = backend.record(args.duration, Arc::clone(&stop), args.device.as_deref())?;

    // Ensure raw mode is restored even if the listener thread is still spinning.
    let _ = terminal::disable_raw_mode();

    sm.handle(DictationEvent::StopRecording);

    let wav_bytes = encode_wav(&audio)?;
    tracing::debug!(bytes = wav_bytes.len(), "WAV encoded");

    eprintln!("Transcribing…");
    let provider = GroqProvider::new(api_key, model);
    let opts = TranscribeOptions {
        language: args.language.clone(),
        ..Default::default()
    };

    let transcript = match provider.transcribe(&wav_bytes, &opts).await {
        Ok(t) => {
            sm.handle(DictationEvent::TranscriptionComplete(t.text.clone()));
            t
        }
        Err(e) => {
            sm.handle(DictationEvent::TranscriptionFailed(e.to_string()));
            return Err(e);
        }
    };

    let sink = output::create_sink(args.output);
    if args.json {
        let json = serde_json::to_string_pretty(&transcript)
            .map_err(voxput_core::errors::VoxputError::Json)?;
        sink.write(&json)?;
    } else {
        sink.write(&transcript.text)?;
    }

    Ok(())
}
