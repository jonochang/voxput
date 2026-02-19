# voxput

Voice-to-text dictation for Linux. Speak into your mic, get text out.

Powered by the [Groq Whisper API](https://console.groq.com/docs/speech-text) for fast transcription.

## Requirements

- Rust toolchain
- A [Groq API key](https://console.groq.com/) (free tier available)
- A microphone accessible via ALSA/PipeWire

## Installation

```bash
git clone https://github.com/jonochang/voxput
cd voxput
cargo build --release
# binary at target/release/voxput
```

## Usage

Set your API key:

```bash
export GROQ_API_KEY=gsk_...
```

### Record and transcribe

```bash
# Record until any key is pressed (default), print to stdout
voxput record

# Record until keypress OR 10 seconds, whichever comes first
voxput record --duration 10

# Copy result to clipboard instead
voxput record --output clipboard

# Both stdout and clipboard
voxput record --output both

# Force English transcription
voxput record --language en

# Use a specific Whisper model
voxput record --model whisper-large-v3

# Print transcript as JSON
voxput record --json

# Use a specific input device
voxput record --device "USB Audio"
```

### List audio devices

```bash
voxput devices
voxput devices --json
```

### Pipe output

```bash
# Append transcription to a file
voxput record >> notes.txt

# Pipe into another tool
voxput record | wl-copy   # Wayland clipboard
voxput record | xclip     # X11 clipboard
```

## Configuration

Optional config file at `~/.config/voxput/config.toml`:

```toml
provider = "groq"

[providers.groq]
api_key_env = "GROQ_API_KEY"   # env var name (default)
model = "whisper-large-v3-turbo"

[audio]
device = "default"
sample_rate = 16000

[output]
target = "stdout"
```

## How it works

```
mic → cpal (audio capture) → WAV encode → Groq Whisper API → stdout / clipboard
```

1. `voxput record` opens the default microphone via [cpal](https://github.com/RustAudio/cpal)
2. Enables terminal raw mode and waits for any keypress (via [crossterm](https://github.com/crossterm-rs/crossterm))
3. Records raw PCM samples at 16 kHz mono until the key is pressed (or `--duration` expires)
4. Encodes them as a 16-bit WAV in memory (no temp files)
5. POSTs the WAV to the Groq Whisper API as `multipart/form-data`
6. Restores the terminal and prints the transcript to stdout (or writes to clipboard)

Status messages go to stderr so stdout is clean for piping.

## Roadmap

- **v0.1 (current):** CLI one-shot mode — `voxput record`
- **v0.2:** Background daemon (`voxputd`) with IPC, push-to-talk support
- **v0.3:** GNOME Shell extension with top-bar indicator
- **v0.4:** Type-at-cursor output, additional providers (OpenAI, local whisper.cpp)
