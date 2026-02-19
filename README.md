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
# CLI: target/release/voxput
# Daemon: target/release/voxputd
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

## Daemon mode (v0.2)

A background daemon (`voxputd`) exposes a D-Bus service on the session bus so
you can start/stop recording without keeping a terminal open.

### Starting the daemon

```bash
# Start manually
voxputd

# Or install as a systemd user service
cp contrib/voxputd.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now voxputd
```

### Push-to-talk via CLI commands

```bash
voxput start    # tell daemon to start recording
voxput stop     # stop recording → triggers transcription
voxput toggle   # start if idle, stop if recording
voxput status   # show current state (idle/recording/transcribing/error)
voxput status --json
```

You can bind `voxput toggle` to a key using any Linux hotkey tool:

```bash
# ~/.config/sxhkd/sxhkdrc (sxhkd)
super + m
    voxput toggle
```

The daemon emits a `StateChanged(state, transcript)` D-Bus signal whenever its
state changes. The last transcript is also available via `voxput status`.

## GNOME Shell extension (v0.3)

The extension lives in `extensions/gnome/`. It adds a top-bar microphone
indicator and lets you configure a keyboard shortcut for push-to-talk entirely
within GNOME Settings.

### Installation

```bash
cd extensions/gnome
make install

# Restart GNOME Shell (Wayland: log out and back in; X11: Alt+F2 → 'r')
gnome-extensions enable voxput@jonochang.github.com
```

### How it works

- The extension connects to `voxputd` on D-Bus at startup.
- Pressing the configured shortcut (default: `Super+M`) calls `Toggle()`.
- The top-bar icon changes colour with each state:
  - **Grey / muted mic** — idle
  - **Red mic** — recording
  - **Yellow spinner** — transcribing
  - **Red warning** — error
- When transcription completes, the result appears in the popup menu and
  optionally as a GNOME notification.
- Configure the shortcut and notification preferences via the extension
  settings (Extensions app → Voxput → ⚙).

## Roadmap

- **v0.1:** CLI one-shot mode — `voxput record`
- **v0.2 (done):** Background daemon (`voxputd`) with D-Bus IPC + push-to-talk
- **v0.3 (done):** GNOME Shell extension with top-bar indicator and shortcut
- **v0.4:** Type-at-cursor output, additional providers (OpenAI, local whisper.cpp)
