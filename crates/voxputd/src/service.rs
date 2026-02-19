use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::OnceCell;
use zbus::{interface, object_server::SignalEmitter};

use voxput_core::{
    audio::{cpal_backend::CpalBackend, wav::encode_wav, AudioBackend, MIN_DURATION_SECS},
    provider::{groq::GroqProvider, TranscribeOptions, TranscriptionProvider},
    state::{DictationEvent, DictationState, DictationStateMachine},
};

/// Service state shared between D-Bus interface methods and background pipeline tasks.
pub(crate) struct ServiceInner {
    sm: Mutex<DictationStateMachine>,
    stop_flag: Arc<AtomicBool>,
    last_transcript: Mutex<String>,
    last_error: Mutex<String>,
    api_key: String,
    model: Option<String>,
    device_name: Option<String>,
    language: Option<String>,
    /// Stored after D-Bus connection is built; used to emit signals from background tasks.
    pub(crate) connection: OnceCell<zbus::Connection>,
}

impl ServiceInner {
    /// Emit a StateChanged D-Bus signal. No-op if the connection is not yet set.
    async fn emit_state(&self, state: &str, transcript: &str) {
        let Some(conn) = self.connection.get() else {
            return;
        };
        match SignalEmitter::new(conn, "/com/github/jonochang/Voxput") {
            Ok(ctxt) => {
                let _ = VoxputService::state_changed(&ctxt, state, transcript).await;
            }
            Err(e) => tracing::error!("Failed to create signal context: {e}"),
        }
    }
}

pub struct VoxputService {
    pub(crate) inner: Arc<ServiceInner>,
}

impl VoxputService {
    pub fn new(
        api_key: String,
        model: Option<String>,
        device_name: Option<String>,
        language: Option<String>,
    ) -> Self {
        Self {
            inner: Arc::new(ServiceInner {
                sm: Mutex::new(DictationStateMachine::new()),
                stop_flag: Arc::new(AtomicBool::new(false)),
                last_transcript: Mutex::new(String::new()),
                last_error: Mutex::new(String::new()),
                api_key,
                model,
                device_name,
                language,
                connection: OnceCell::new(),
            }),
        }
    }

    pub fn inner_arc(&self) -> Arc<ServiceInner> {
        Arc::clone(&self.inner)
    }

    /// Launch the recording+transcription pipeline in a background task.
    fn spawn_pipeline(&self) {
        let inner = Arc::clone(&self.inner);
        tokio::spawn(async move {
            run_pipeline(inner).await;
        });
    }
}

#[interface(name = "com.github.jonochang.Voxput1")]
impl VoxputService {
    /// Begin recording audio. No-op if already recording or transcribing.
    async fn start_recording(&self) -> zbus::fdo::Result<()> {
        let state = self.inner.sm.lock().unwrap().state();
        if matches!(state, DictationState::Recording | DictationState::Transcribing) {
            return Ok(());
        }
        {
            let mut sm = self.inner.sm.lock().unwrap();
            if sm.state() == DictationState::Error {
                sm.handle(DictationEvent::Reset);
            }
            sm.handle(DictationEvent::StartRecording);
        }
        self.inner.stop_flag.store(false, Ordering::SeqCst);
        self.inner.emit_state("recording", "").await;
        self.spawn_pipeline();
        Ok(())
    }

    /// Stop an in-progress recording (sets the stop flag; pipeline continues to transcribe).
    async fn stop_recording(&self) -> zbus::fdo::Result<()> {
        if self.inner.sm.lock().unwrap().state() == DictationState::Recording {
            self.inner.stop_flag.store(true, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Start recording if idle; stop recording if recording. No-op while transcribing.
    async fn toggle(&self) -> zbus::fdo::Result<()> {
        let state = self.inner.sm.lock().unwrap().state();
        match state {
            DictationState::Idle | DictationState::Error => self.start_recording().await,
            DictationState::Recording => self.stop_recording().await,
            DictationState::Transcribing => Ok(()),
        }
    }

    /// Return (state, last_transcript, last_error) strings.
    async fn get_status(&self) -> zbus::fdo::Result<(String, String, String)> {
        let state = self.inner.sm.lock().unwrap().state().to_string();
        let transcript = self.inner.last_transcript.lock().unwrap().clone();
        let error = self.inner.last_error.lock().unwrap().clone();
        Ok((state, transcript, error))
    }

    /// Emitted whenever the daemon's state changes.
    /// `state` is one of: "idle", "recording", "transcribing", "error".
    /// `transcript` is the completed text (only set when state returns to "idle").
    #[zbus(signal)]
    async fn state_changed(
        ctxt: &SignalEmitter<'_>,
        state: &str,
        transcript: &str,
    ) -> zbus::Result<()>;
}

// ---------------------------------------------------------------------------
// Background recording + transcription pipeline
// ---------------------------------------------------------------------------

async fn run_pipeline(inner: Arc<ServiceInner>) {
    tracing::info!("Pipeline: recording");

    let stop_flag = Arc::clone(&inner.stop_flag);
    let device = inner.device_name.clone();

    // 1. Record (blocking)
    let audio = match tokio::task::spawn_blocking(move || {
        CpalBackend.record(0.0, stop_flag, device.as_deref())
    })
    .await
    {
        Ok(Ok(a)) => a,
        Ok(Err(e)) => {
            pipeline_error(&inner, &e.to_string()).await;
            return;
        }
        Err(e) => {
            pipeline_error(&inner, &e.to_string()).await;
            return;
        }
    };

    // 2. Duration guard
    if audio.duration_secs() < MIN_DURATION_SECS {
        pipeline_error(
            &inner,
            &format!(
                "Recording too short ({:.3}s); hold longer before releasing",
                audio.duration_secs()
            ),
        )
        .await;
        return;
    }

    // 3. Encode WAV in memory
    let wav = match encode_wav(&audio) {
        Ok(w) => w,
        Err(e) => {
            pipeline_error(&inner, &e.to_string()).await;
            return;
        }
    };

    // 4. Advance state machine: Recording → Transcribing
    {
        let mut sm = inner.sm.lock().unwrap();
        sm.handle(DictationEvent::StopRecording);
    }
    inner.emit_state("transcribing", "").await;
    tracing::info!("Pipeline: transcribing");

    // 5. Transcribe
    let provider = GroqProvider::new(inner.api_key.clone(), inner.model.clone());
    let mut opts = TranscribeOptions::default();
    opts.language = inner.language.clone();

    let transcript_text = match provider.transcribe(&wav, &opts).await {
        Ok(t) => t.text,
        Err(e) => {
            {
                let mut sm = inner.sm.lock().unwrap();
                sm.handle(DictationEvent::TranscriptionFailed(e.to_string()));
            }
            *inner.last_error.lock().unwrap() = e.to_string();
            inner.emit_state("error", "").await;
            tracing::error!("Transcription failed: {e}");
            return;
        }
    };

    // 6. Complete: Transcribing → Idle
    {
        let mut sm = inner.sm.lock().unwrap();
        sm.handle(DictationEvent::TranscriptionComplete(transcript_text.clone()));
    }
    *inner.last_transcript.lock().unwrap() = transcript_text.clone();
    inner.emit_state("idle", &transcript_text).await;
    tracing::info!("Pipeline: done — {transcript_text}");
}

async fn pipeline_error(inner: &Arc<ServiceInner>, error: &str) {
    tracing::error!("Pipeline error: {error}");
    {
        let mut sm = inner.sm.lock().unwrap();
        match sm.state() {
            DictationState::Recording => {
                sm.handle(DictationEvent::StopRecording);
                sm.handle(DictationEvent::TranscriptionFailed(error.to_string()));
            }
            DictationState::Transcribing => {
                sm.handle(DictationEvent::TranscriptionFailed(error.to_string()));
            }
            _ => {
                sm.handle(DictationEvent::Reset);
            }
        }
    }
    *inner.last_error.lock().unwrap() = error.to_string();
    inner.emit_state("error", "").await;
}
