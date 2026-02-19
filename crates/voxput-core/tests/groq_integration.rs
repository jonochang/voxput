/// Integration tests for the Groq Whisper provider.
///
/// Requires a valid `GROQ_API_KEY` env var.  Tests are skipped automatically
/// when the key is absent so CI without credentials stays green.
///
/// Run with:
///   GROQ_API_KEY=gsk_... cargo test --test groq_integration -- --nocapture

use voxput_core::provider::groq::GroqProvider;
use voxput_core::provider::{TranscribeOptions, TranscriptionProvider};

/// Path to the espeak-generated voice fixture shipped with the repo.
const VOICE_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/hello_world.wav"
);

/// Return the Groq API key or `None` if not set (silently skips the test).
fn api_key() -> Option<String> {
    std::env::var("GROQ_API_KEY").ok().filter(|k| !k.is_empty())
}

#[tokio::test]
async fn transcribes_voice_fixture_to_expected_text() {
    let Some(key) = api_key() else {
        eprintln!("GROQ_API_KEY not set — skipping Groq integration test");
        return;
    };

    let wav_bytes = std::fs::read(VOICE_FIXTURE)
        .unwrap_or_else(|e| panic!("Could not read voice fixture at {VOICE_FIXTURE}: {e}"));

    let provider = GroqProvider::new(key, None);
    let transcript = provider
        .transcribe(&wav_bytes, &TranscribeOptions::default())
        .await
        .expect("Groq transcription should succeed");

    let text = transcript.text.to_lowercase();
    eprintln!("Transcript: {:?}", transcript.text);

    // espeak says "hello world this is a voice dictation test"
    // Groq may capitalise or add punctuation, so check lowercase keywords
    assert!(
        text.contains("hello") && text.contains("world"),
        "Expected transcript to contain 'hello world', got: {:?}",
        transcript.text
    );
}

#[tokio::test]
async fn invalid_api_key_returns_provider_error() {
    let wav_bytes = std::fs::read(VOICE_FIXTURE)
        .unwrap_or_else(|e| panic!("Could not read voice fixture at {VOICE_FIXTURE}: {e}"));

    let provider = GroqProvider::new("bad-key-intentionally-invalid".into(), None);
    let err = provider
        .transcribe(&wav_bytes, &TranscribeOptions::default())
        .await
        .expect_err("Should fail with a bad API key");

    let msg = err.to_string();
    assert!(
        msg.contains("401") || msg.contains("invalid") || msg.contains("API key"),
        "Expected 401/invalid-key error, got: {msg}"
    );
}
