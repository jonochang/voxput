use clap::Args;
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
    /// Recording duration in seconds
    #[arg(long, short, default_value = "5")]
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

    // Load config and resolve API key
    let config = config::load_config()?;
    let api_key = config.api_key()?;
    let model = args.model.clone().or(config.model.clone());

    // Record
    sm.handle(DictationEvent::StartRecording);
    eprintln!("Recording for {:.1}s… (speak now)", args.duration);
    let backend = CpalBackend;
    let audio = backend.record(args.duration, args.device.as_deref())?;
    sm.handle(DictationEvent::StopRecording);

    // Encode
    let wav_bytes = encode_wav(&audio)?;
    tracing::debug!(bytes = wav_bytes.len(), "WAV encoded");

    // Transcribe
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

    // Output
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
