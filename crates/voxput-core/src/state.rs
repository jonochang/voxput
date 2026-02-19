use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationState {
    Idle,
    Recording,
    Transcribing,
    Error,
}

impl fmt::Display for DictationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Recording => write!(f, "recording"),
            Self::Transcribing => write!(f, "transcribing"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DictationEvent {
    StartRecording,
    StopRecording,
    TranscriptionComplete(String),
    TranscriptionFailed(String),
    Reset,
}

pub struct DictationStateMachine {
    state: DictationState,
    last_error: Option<String>,
    last_transcript: Option<String>,
}

impl Default for DictationStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl DictationStateMachine {
    pub fn new() -> Self {
        Self {
            state: DictationState::Idle,
            last_error: None,
            last_transcript: None,
        }
    }

    pub fn state(&self) -> DictationState {
        self.state
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn last_transcript(&self) -> Option<&str> {
        self.last_transcript.as_deref()
    }

    pub fn handle(&mut self, event: DictationEvent) -> DictationState {
        match (&self.state, &event) {
            (DictationState::Idle, DictationEvent::StartRecording) => {
                self.state = DictationState::Recording;
            }
            (DictationState::Recording, DictationEvent::StopRecording) => {
                self.state = DictationState::Transcribing;
            }
            (DictationState::Transcribing, DictationEvent::TranscriptionComplete(text)) => {
                self.last_transcript = Some(text.clone());
                self.last_error = None;
                self.state = DictationState::Idle;
            }
            (DictationState::Transcribing, DictationEvent::TranscriptionFailed(err)) => {
                self.last_error = Some(err.clone());
                self.state = DictationState::Error;
            }
            (_, DictationEvent::Reset) => {
                self.last_error = None;
                self.state = DictationState::Idle;
            }
            (current, event) => {
                tracing::warn!(
                    state = %current,
                    "Ignored invalid state transition: {:?}",
                    event
                );
            }
        }
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_idle() {
        let sm = DictationStateMachine::new();
        assert_eq!(sm.state(), DictationState::Idle);
    }

    #[test]
    fn idle_to_recording() {
        let mut sm = DictationStateMachine::new();
        let s = sm.handle(DictationEvent::StartRecording);
        assert_eq!(s, DictationState::Recording);
    }

    #[test]
    fn recording_to_transcribing() {
        let mut sm = DictationStateMachine::new();
        sm.handle(DictationEvent::StartRecording);
        let s = sm.handle(DictationEvent::StopRecording);
        assert_eq!(s, DictationState::Transcribing);
    }

    #[test]
    fn transcription_complete_returns_to_idle() {
        let mut sm = DictationStateMachine::new();
        sm.handle(DictationEvent::StartRecording);
        sm.handle(DictationEvent::StopRecording);
        let s = sm.handle(DictationEvent::TranscriptionComplete("hello".into()));
        assert_eq!(s, DictationState::Idle);
        assert_eq!(sm.last_transcript(), Some("hello"));
    }

    #[test]
    fn transcription_failed_goes_to_error() {
        let mut sm = DictationStateMachine::new();
        sm.handle(DictationEvent::StartRecording);
        sm.handle(DictationEvent::StopRecording);
        let s = sm.handle(DictationEvent::TranscriptionFailed("oops".into()));
        assert_eq!(s, DictationState::Error);
        assert_eq!(sm.last_error(), Some("oops"));
    }

    #[test]
    fn reset_from_any_state() {
        let mut sm = DictationStateMachine::new();
        sm.handle(DictationEvent::StartRecording);
        sm.handle(DictationEvent::StopRecording);
        sm.handle(DictationEvent::TranscriptionFailed("err".into()));
        assert_eq!(sm.state(), DictationState::Error);
        let s = sm.handle(DictationEvent::Reset);
        assert_eq!(s, DictationState::Idle);
        assert!(sm.last_error().is_none());
    }

    #[test]
    fn invalid_transition_does_not_panic() {
        let mut sm = DictationStateMachine::new();
        // StopRecording from Idle is invalid — should not panic
        let s = sm.handle(DictationEvent::StopRecording);
        assert_eq!(s, DictationState::Idle);
    }
}
