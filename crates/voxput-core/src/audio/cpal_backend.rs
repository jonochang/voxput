use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::audio::{AudioBackend, AudioData, DeviceInfo};
use crate::errors::{Result, VoxputError};

pub struct CpalBackend;

impl AudioBackend for CpalBackend {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let host = cpal::default_host();
        let default_name = host
            .default_input_device()
            .and_then(|d| d.name().ok())
            .unwrap_or_default();

        let devices = host
            .input_devices()
            .map_err(|e| VoxputError::Audio(format!("Failed to enumerate devices: {e}")))?
            .filter_map(|d| d.name().ok().map(|name| DeviceInfo {
                is_default: name == default_name,
                name,
            }))
            .collect();

        Ok(devices)
    }

    fn record(
        &self,
        duration_secs: f32,
        stop: Arc<AtomicBool>,
        device_name: Option<&str>,
    ) -> Result<AudioData> {
        let host = cpal::default_host();

        let device = match device_name {
            Some(name) => host
                .input_devices()
                .map_err(|e| VoxputError::Audio(format!("Failed to enumerate devices: {e}")))?
                .find(|d| d.name().ok().as_deref() == Some(name))
                .ok_or_else(|| VoxputError::Audio(format!("Device '{name}' not found")))?,
            None => host
                .default_input_device()
                .ok_or(VoxputError::NoDevice)?,
        };

        // Prefer 16 kHz mono; fall back to device default.
        let config = select_config(&device)?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let samples_writer = Arc::clone(&samples);

        let err_flag: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let err_writer = Arc::clone(&err_flag);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut buf = samples_writer.lock().unwrap();
                    if channels == 1 {
                        buf.extend_from_slice(data);
                    } else {
                        for chunk in data.chunks_exact(channels as usize) {
                            let mono = chunk.iter().sum::<f32>() / channels as f32;
                            buf.push(mono);
                        }
                    }
                },
                move |e| {
                    *err_writer.lock().unwrap() = Some(e.to_string());
                },
                None,
            )
            .map_err(|e| VoxputError::Audio(format!("Failed to build input stream: {e}")))?;

        stream
            .play()
            .map_err(|e| VoxputError::Audio(format!("Failed to start stream: {e}")))?;

        // Poll every 50 ms; exit when stop flag is set or duration expires.
        let max_duration = if duration_secs > 0.0 {
            Some(Duration::from_secs_f32(duration_secs))
        } else {
            None
        };
        let start = Instant::now();

        loop {
            std::thread::sleep(Duration::from_millis(50));
            if stop.load(Ordering::Relaxed) {
                break;
            }
            if let Some(max) = max_duration {
                if start.elapsed() >= max {
                    stop.store(true, Ordering::Relaxed); // tell listener to exit too
                    break;
                }
            }
        }

        drop(stream);

        if let Some(err) = err_flag.lock().unwrap().take() {
            return Err(VoxputError::Audio(format!("Stream error during recording: {err}")));
        }

        let raw = Arc::try_unwrap(samples)
            .map_err(|_| VoxputError::Audio("Failed to collect samples".into()))?
            .into_inner()
            .unwrap();

        Ok(AudioData {
            samples: raw,
            sample_rate,
            channels: 1,
        })
    }
}

/// Try to get a 16 kHz mono f32 config; fall back to device default.
fn select_config(device: &cpal::Device) -> Result<cpal::SupportedStreamConfig> {
    let supported = device
        .supported_input_configs()
        .map_err(|e| VoxputError::Audio(format!("Failed to query configs: {e}")))?;

    for range in supported {
        if range.sample_format() == cpal::SampleFormat::F32
            && range.min_sample_rate().0 <= 16000
            && range.max_sample_rate().0 >= 16000
        {
            return Ok(range.with_sample_rate(cpal::SampleRate(16000)));
        }
    }

    device
        .default_input_config()
        .map_err(|e| VoxputError::Audio(format!("Failed to get default config: {e}")))
}
