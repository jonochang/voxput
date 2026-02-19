pub mod cpal_backend;
pub mod wav;

use crate::errors::Result;

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

    /// Record audio for the specified duration in seconds.
    fn record(&self, duration_secs: f32, device_name: Option<&str>) -> Result<AudioData>;
}
