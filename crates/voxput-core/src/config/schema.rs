use serde::Deserialize;

/// TOML-deserializable config file format.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileConfig {
    #[serde(default)]
    pub provider: Option<String>,

    #[serde(default)]
    pub providers: ProvidersConfig,

    #[serde(default)]
    pub audio: AudioConfig,

    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub groq: Option<GroqConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GroqConfig {
    /// Name of the env var that holds the API key (default: "GROQ_API_KEY").
    pub api_key_env: Option<String>,
    /// Directly embedded API key (not recommended; prefer env var).
    pub api_key: Option<String>,
    /// Model name (e.g. "whisper-large-v3-turbo").
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AudioConfig {
    /// Preferred input device name.
    pub device: Option<String>,
    /// Sample rate in Hz (default 16000).
    pub sample_rate: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OutputConfig {
    /// Default output target: "stdout", "clipboard", or "both".
    pub target: Option<String>,
}

impl FileConfig {
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_toml_deserialises_to_defaults() {
        let cfg = FileConfig::from_toml("").expect("empty TOML should parse");
        assert!(cfg.provider.is_none());
        assert!(cfg.providers.groq.is_none());
    }

    #[test]
    fn full_toml_round_trips() {
        let toml = r#"
provider = "groq"

[providers.groq]
api_key_env = "GROQ_API_KEY"
model = "whisper-large-v3-turbo"

[audio]
device = "default"
sample_rate = 16000

[output]
target = "stdout"
"#;
        let cfg = FileConfig::from_toml(toml).expect("TOML should parse");
        assert_eq!(cfg.provider.as_deref(), Some("groq"));
        let groq = cfg.providers.groq.expect("groq config should be present");
        assert_eq!(groq.api_key_env.as_deref(), Some("GROQ_API_KEY"));
        assert_eq!(groq.model.as_deref(), Some("whisper-large-v3-turbo"));
        assert_eq!(cfg.audio.sample_rate, Some(16000));
        assert_eq!(cfg.output.target.as_deref(), Some("stdout"));
    }

    #[test]
    fn partial_toml_works() {
        let toml = r#"provider = "groq""#;
        let cfg = FileConfig::from_toml(toml).expect("partial TOML should parse");
        assert_eq!(cfg.provider.as_deref(), Some("groq"));
        assert!(cfg.providers.groq.is_none());
    }
}
