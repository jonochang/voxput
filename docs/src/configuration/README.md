# Configuration

Voxput uses an optional config file at `~/.config/voxput/config.toml`.

## Config file format

```toml
provider = "groq"

[providers.groq]
api_key_env = "GROQ_API_KEY"   # env var name (default)
model = "whisper-large-v3-turbo"

[audio]
device = "default"
sample_rate = 16000

[output]
# "stdout" (default for voxput record), "clipboard", or "both"
# voxputd always copies to clipboard regardless of this setting,
# unless you set "both" to also log transcripts to stdout.
target = "stdout"
```

## Resolution order

Settings are resolved in this order (later overrides earlier):

1. Built-in defaults
2. Config file (`~/.config/voxput/config.toml`)
3. Environment variables (`GROQ_API_KEY`, `VOXPUT_MODEL`)
4. CLI flags (`--model`, `--device`, etc.)

## Environment variables

| Variable | Description |
|----------|-------------|
| `GROQ_API_KEY` | Groq API key for transcription |
| `VOXPUT_MODEL` | Override the Whisper model |
| `RUST_LOG` | Set log level (e.g., `debug`, `info`) |
