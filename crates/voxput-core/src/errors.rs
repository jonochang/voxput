use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum VoxputError {
    #[error("Audio error: {0}")]
    #[diagnostic(code(voxput::audio))]
    Audio(String),

    #[error("No audio input device available")]
    #[diagnostic(
        code(voxput::no_device),
        help("Check that a microphone is connected and accessible")
    )]
    NoDevice,

    #[error("Transcription provider error: {0}")]
    #[diagnostic(code(voxput::provider))]
    Provider(String),

    #[error("API key not found: set {env_var} or add to ~/.config/voxput/config.toml")]
    #[diagnostic(code(voxput::missing_api_key))]
    MissingApiKey { env_var: String },

    #[error("Configuration error: {0}")]
    #[diagnostic(code(voxput::config))]
    Config(String),

    #[error("Output error: {0}")]
    #[diagnostic(code(voxput::output))]
    Output(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VoxputError>;
