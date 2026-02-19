pub mod daemon;
pub mod devices;
pub mod record;

use clap::{Parser, Subcommand};
use voxput_core::errors::Result;

#[derive(Debug, Parser)]
#[command(
    name = "voxput",
    version,
    about = "Voice-to-text dictation tool",
    long_about = "Record audio from your microphone and transcribe it using the Groq Whisper API."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Record audio and transcribe to text (standalone, no daemon required)
    Record(record::RecordArgs),

    /// List available audio input devices
    Devices(devices::DevicesArgs),

    /// Tell the voxputd daemon to start recording
    Start(daemon::StartArgs),

    /// Tell the voxputd daemon to stop recording (begins transcription)
    Stop(daemon::StopArgs),

    /// Toggle recording on the voxputd daemon (start if idle, stop if recording)
    Toggle(daemon::ToggleArgs),

    /// Show the voxputd daemon's current state
    Status(daemon::StatusArgs),
}

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Record(args) => record::run(&args).await,
        Commands::Devices(args) => devices::run(&args),
        Commands::Start(args) => daemon::run_start(&args).await,
        Commands::Stop(args) => daemon::run_stop(&args).await,
        Commands::Toggle(args) => daemon::run_toggle(&args).await,
        Commands::Status(args) => daemon::run_status(&args).await,
    }
}
