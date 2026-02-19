pub mod groq;

use crate::errors::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct TranscribeOptions {
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub temperature: Option<f32>,
}

#[async_trait]
pub trait TranscriptionProvider: Send + Sync {
    async fn transcribe(&self, audio_wav: &[u8], opts: &TranscribeOptions) -> Result<Transcript>;
    fn name(&self) -> &str;
}
