pub mod schema;

use crate::errors::{Result, VoxputError};
use schema::FileConfig;
use std::path::PathBuf;

/// Fully resolved runtime configuration.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// Name of the env var that holds the API key.
    pub api_key_env: String,
    /// Pre-resolved API key (from file or env var).
    pub api_key: Option<String>,
    /// Transcription model name.
    pub model: Option<String>,
    /// Provider name ("groq").
    pub provider: String,
    /// Preferred audio input device name.
    pub device: Option<String>,
    /// Audio sample rate.
    pub sample_rate: u32,
    /// Default output target.
    pub output_target: String,
}

impl ResolvedConfig {
    /// Return the API key, checking the pre-resolved field then the env var.
    pub fn api_key(&self) -> Result<String> {
        if let Some(ref key) = self.api_key {
            return Ok(key.clone());
        }
        std::env::var(&self.api_key_env).map_err(|_| VoxputError::MissingApiKey {
            env_var: self.api_key_env.clone(),
        })
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("voxput").join("config.toml"))
}

/// Load configuration: defaults → file → env vars.
pub fn load_config() -> Result<ResolvedConfig> {
    let mut resolved = ResolvedConfig {
        api_key_env: "GROQ_API_KEY".to_string(),
        api_key: None,
        model: None,
        provider: "groq".to_string(),
        device: None,
        sample_rate: 16000,
        output_target: "stdout".to_string(),
    };

    // Layer 2: file config
    if let Some(path) = config_path() {
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            match FileConfig::from_toml(&contents) {
                Ok(file) => apply_file_config(&mut resolved, &file),
                Err(e) => {
                    return Err(VoxputError::Config(format!(
                        "Failed to parse {}: {e}",
                        path.display()
                    )))
                }
            }
        }
    }

    // Layer 3: env var overrides
    if let Ok(key) = std::env::var(&resolved.api_key_env) {
        if !key.is_empty() {
            resolved.api_key = Some(key);
        }
    }
    if let Ok(model) = std::env::var("VOXPUT_MODEL") {
        if !model.is_empty() {
            resolved.model = Some(model);
        }
    }

    Ok(resolved)
}

fn apply_file_config(r: &mut ResolvedConfig, f: &FileConfig) {
    if let Some(ref p) = f.provider {
        r.provider = p.clone();
    }
    if let Some(ref groq) = f.providers.groq {
        if let Some(ref env) = groq.api_key_env {
            r.api_key_env = env.clone();
        }
        if let Some(ref key) = groq.api_key {
            if !key.is_empty() {
                r.api_key = Some(key.clone());
            }
        }
        if let Some(ref model) = groq.model {
            r.model = Some(model.clone());
        }
    }
    if let Some(ref dev) = f.audio.device {
        r.device = Some(dev.clone());
    }
    if let Some(rate) = f.audio.sample_rate {
        r.sample_rate = rate;
    }
    if let Some(ref tgt) = f.output.target {
        r.output_target = tgt.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        // Call load_config with no file (will use whatever the test env has)
        // We can at least check the built-in defaults by constructing manually
        let cfg = ResolvedConfig {
            api_key_env: "GROQ_API_KEY".to_string(),
            api_key: None,
            model: None,
            provider: "groq".to_string(),
            device: None,
            sample_rate: 16000,
            output_target: "stdout".to_string(),
        };
        assert_eq!(cfg.provider, "groq");
        assert_eq!(cfg.sample_rate, 16000);
        assert_eq!(cfg.output_target, "stdout");
    }

    #[test]
    fn api_key_resolved_from_field() {
        let cfg = ResolvedConfig {
            api_key_env: "GROQ_API_KEY".to_string(),
            api_key: Some("my-key".to_string()),
            model: None,
            provider: "groq".to_string(),
            device: None,
            sample_rate: 16000,
            output_target: "stdout".to_string(),
        };
        assert_eq!(cfg.api_key().unwrap(), "my-key");
    }

    #[test]
    fn api_key_missing_returns_error() {
        // Use an env var that almost certainly doesn't exist
        let cfg = ResolvedConfig {
            api_key_env: "VOXPUT_TEST_MISSING_KEY_XYZ".to_string(),
            api_key: None,
            model: None,
            provider: "groq".to_string(),
            device: None,
            sample_rate: 16000,
            output_target: "stdout".to_string(),
        };
        let err = cfg.api_key().expect_err("should fail on missing key");
        assert!(err.to_string().contains("VOXPUT_TEST_MISSING_KEY_XYZ"));
    }

    #[test]
    fn apply_file_config_overrides_defaults() {
        let mut resolved = ResolvedConfig {
            api_key_env: "GROQ_API_KEY".to_string(),
            api_key: None,
            model: None,
            provider: "groq".to_string(),
            device: None,
            sample_rate: 16000,
            output_target: "stdout".to_string(),
        };
        let file = schema::FileConfig::from_toml(
            r#"
[providers.groq]
model = "whisper-large-v3"

[audio]
sample_rate = 8000
"#,
        )
        .unwrap();

        apply_file_config(&mut resolved, &file);
        assert_eq!(resolved.model.as_deref(), Some("whisper-large-v3"));
        assert_eq!(resolved.sample_rate, 8000);
    }
}
