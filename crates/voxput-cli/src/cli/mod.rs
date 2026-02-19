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
    /// Record audio and transcribe to text
    Record(record::RecordArgs),
    /// List available audio input devices
    Devices(devices::DevicesArgs),
}

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Record(args) => record::run(&args).await,
        Commands::Devices(args) => devices::run(&args),
    }
}
