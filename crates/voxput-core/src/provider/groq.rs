use async_trait::async_trait;
use reqwest::multipart;
use serde::Deserialize;

use crate::errors::{Result, VoxputError};
use crate::provider::{TranscribeOptions, Transcript, TranscriptionProvider};

const GROQ_TRANSCRIPTION_URL: &str =
    "https://api.groq.com/openai/v1/audio/transcriptions";

pub struct GroqProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
    /// Override base URL for testing.
    base_url: String,
}

impl GroqProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "whisper-large-v3-turbo".into()),
            client: reqwest::Client::new(),
            base_url: GROQ_TRANSCRIPTION_URL.to_string(),
        }
    }

    /// Create with a custom base URL (for tests / mock servers).
    pub fn with_base_url(api_key: String, model: Option<String>, base_url: String) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "whisper-large-v3-turbo".into()),
            client: reqwest::Client::new(),
            base_url,
        }
    }
}

/// Groq error response body.
#[derive(Debug, Deserialize)]
struct GroqErrorBody {
    error: GroqErrorDetail,
}

#[derive(Debug, Deserialize)]
struct GroqErrorDetail {
    message: String,
}

/// Minimal response shape — Groq returns `{ "text": "..." }`.
#[derive(Debug, Deserialize)]
struct GroqResponse {
    text: String,
}

#[async_trait]
impl TranscriptionProvider for GroqProvider {
    fn name(&self) -> &str {
        "groq"
    }

    async fn transcribe(&self, audio_wav: &[u8], opts: &TranscribeOptions) -> Result<Transcript> {
        let audio_part = multipart::Part::bytes(audio_wav.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| VoxputError::Provider(format!("MIME error: {e}")))?;

        let mut form = multipart::Form::new()
            .part("file", audio_part)
            .text("model", self.model.clone())
            .text("response_format", "json");

        if let Some(ref lang) = opts.language {
            form = form.text("language", lang.clone());
        }
        if let Some(ref prompt) = opts.prompt {
            form = form.text("prompt", prompt.clone());
        }
        if let Some(temp) = opts.temperature {
            form = form.text("temperature", temp.to_string());
        }

        let resp = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            let hint = match status.as_u16() {
                401 => " (invalid API key)",
                413 => " (audio file too large; max 25 MB)",
                429 => " (rate limited; wait and retry)",
                _ => "",
            };
            // Try to extract API error message
            let body = resp.text().await.unwrap_or_default();
            let api_msg = serde_json::from_str::<GroqErrorBody>(&body)
                .map(|e| e.error.message)
                .unwrap_or_else(|_| body.clone());
            return Err(VoxputError::Provider(format!(
                "HTTP {status}{hint}: {api_msg}"
            )));
        }

        let groq_resp: GroqResponse = resp.json().await?;
        Ok(Transcript {
            text: groq_resp.text,
            language: None,
            duration: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::ServerOpts;

    fn dummy_wav() -> Vec<u8> {
        // Minimal valid WAV: 44-byte header + 0 data bytes
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes()); // chunk size
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&16000u32.to_le_bytes()); // sample rate
        wav.extend_from_slice(&32000u32.to_le_bytes()); // byte rate
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&0u32.to_le_bytes()); // data size
        wav
    }

    #[tokio::test]
    async fn transcribe_success() {
        let mut server = mockito::Server::new_with_opts_async(ServerOpts::default()).await;
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"text":"hello world"}"#)
            .create_async()
            .await;

        let provider = GroqProvider::with_base_url(
            "test-key".into(),
            None,
            server.url() + "/",
        );
        let result = provider
            .transcribe(&dummy_wav(), &TranscribeOptions::default())
            .await
            .expect("transcribe should succeed");

        assert_eq!(result.text, "hello world");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn transcribe_401_returns_provider_error() {
        let mut server = mockito::Server::new_with_opts_async(ServerOpts::default()).await;
        server
            .mock("POST", "/")
            .with_status(401)
            .with_body(r#"{"error":{"message":"Invalid API key"}}"#)
            .create_async()
            .await;

        let provider = GroqProvider::with_base_url(
            "bad-key".into(),
            None,
            server.url() + "/",
        );
        let err = provider
            .transcribe(&dummy_wav(), &TranscribeOptions::default())
            .await
            .expect_err("should fail on 401");

        let msg = err.to_string();
        assert!(msg.contains("401"), "Expected 401 in: {msg}");
    }

    #[tokio::test]
    async fn transcribe_with_language_option() {
        let mut server = mockito::Server::new_with_opts_async(ServerOpts::default()).await;
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(r#"{"text":"bonjour"}"#)
            .create_async()
            .await;

        let provider = GroqProvider::with_base_url(
            "test-key".into(),
            None,
            server.url() + "/",
        );
        let opts = TranscribeOptions {
            language: Some("fr".into()),
            ..Default::default()
        };
        let result = provider
            .transcribe(&dummy_wav(), &opts)
            .await
            .expect("transcribe should succeed");

        assert_eq!(result.text, "bonjour");
        mock.assert_async().await;
    }
}
