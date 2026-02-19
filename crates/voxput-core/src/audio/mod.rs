pub mod cpal_backend;
pub mod wav;

use crate::errors::Result;
use std::sync::{atomic::AtomicBool, Arc};

/// Minimum audio duration accepted by transcription providers (seconds).
pub const MIN_DURATION_SECS: f32 = 0.1;

/// Captured audio data ready for transcription.
#[derive(Debug, Clone)]
pub struct AudioData {
    /// Raw PCM samples (f32, mono).
    pub samples: Vec<f32>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels (always 1 for mono).
    pub channels: u16,
}

impl AudioData {
    /// Duration of the captured audio in seconds.
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.samples.len() as f32 / self.sample_rate as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_secs_correct() {
        let audio = AudioData { samples: vec![0.0; 1600], sample_rate: 16000, channels: 1 };
        assert!((audio.duration_secs() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn duration_secs_zero_for_empty() {
        let audio = AudioData { samples: vec![], sample_rate: 16000, channels: 1 };
        assert_eq!(audio.duration_secs(), 0.0);
    }
}

/// Information about an audio input device.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
}

/// Trait for audio capture backends.
pub trait AudioBackend: Send + Sync {
    /// List available input devices.
    fn list_devices(&self) -> Result<Vec<DeviceInfo>>;

    /// Record audio until `stop` is set or `duration_secs` elapses, whichever comes first.
    /// A `duration_secs` of `0.0` means no time limit — only the stop flag ends recording.
    fn record(
        &self,
        duration_secs: f32,
        stop: Arc<AtomicBool>,
        device_name: Option<&str>,
    ) -> Result<AudioData>;
}
