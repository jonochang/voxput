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

# Install both binaries to ~/.local/bin (or any directory on your $PATH)
install -Dm755 target/release/voxput   ~/.local/bin/voxput
install -Dm755 target/release/voxputd  ~/.local/bin/voxputd
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

`voxputd` is a background daemon that exposes a D-Bus service on the session
bus. It holds a persistent microphone stream and lets you start/stop recording
from any other program without keeping a terminal open — ideal for hotkey
bindings and the GNOME extension.

### 1. Make the API key available to the daemon

The daemon reads `GROQ_API_KEY` from the environment, or from
`~/.config/voxput/config.toml`. The simplest approach is to add it to your
session environment so it is inherited by all user services:

```bash
# Option A — persist in systemd user environment (survives reboots)
systemctl --user set-environment GROQ_API_KEY=gsk_...

# Option B — add to ~/.config/voxput/config.toml
mkdir -p ~/.config/voxput
cat > ~/.config/voxput/config.toml <<'EOF'
[providers.groq]
api_key = "gsk_..."
EOF
```

### 2. Install and start the daemon

```bash
# Copy the systemd service unit
cp contrib/voxputd.service ~/.config/systemd/user/

# Tell systemd to pick it up, then enable and start
systemctl --user daemon-reload
systemctl --user enable --now voxputd

# Confirm it is running
systemctl --user status voxputd
```

To run the daemon manually (foreground, logs to stderr) instead of via
systemd:

```bash
GROQ_API_KEY=gsk_... voxputd
```

### 3. Verify the connection

```bash
voxput status
# state:      idle
```

If the daemon is not running you will see a hint:

```
Error: Could not connect to voxputd
Hint: start the daemon with `voxputd` or `systemctl --user start voxputd`
```

### 4. Control recording via CLI

```bash
voxput start    # begin recording
voxput stop     # stop recording → transcription runs in the daemon
voxput toggle   # start if idle, stop if recording
voxput status   # show state (idle / recording / transcribing / error)
voxput status --json
```

### 5. Check daemon logs

```bash
journalctl --user -u voxputd -f
```

Set `RUST_LOG=debug` in the service environment for verbose output:

```bash
systemctl --user set-environment RUST_LOG=debug
systemctl --user restart voxputd
```

### Push-to-talk with a hotkey tool

Any tool that can run a shell command on a key event works:

```bash
# sxhkd (~/.config/sxhkd/sxhkdrc)
super + m
    voxput toggle

# xbindkeys (~/.xbindkeysrc)
"voxput toggle"
  Mod4 + m
```

The daemon emits a `StateChanged(state, transcript)` D-Bus signal on every
transition, which you can watch with:

```bash
dbus-monitor "type='signal',interface='com.github.jonochang.Voxput1'"
```

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
