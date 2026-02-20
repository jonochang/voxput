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
# "stdout" (default for voxput record), "clipboard", or "both"
# voxputd always copies to clipboard regardless of this setting,
# unless you set "both" to also log transcripts to stdout.
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

When transcription finishes the daemon copies the result to the clipboard
automatically. Switch to your target window and press `Ctrl+V` to paste.

### 5. Check daemon logs

```bash
journalctl --user -u voxputd -f
```

Set `RUST_LOG=debug` in the service environment for verbose output:

```bash
systemctl --user set-environment RUST_LOG=debug
systemctl --user restart voxputd
```

### Push-to-talk with a hotkey tool (non-GNOME)

If you are not using the GNOME Shell extension, any tool that can run a
shell command on a key event works:

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

The extension lives in `extensions/gnome/`. It adds a microphone indicator to
the top bar and a configurable keyboard shortcut for hands-free push-to-talk.
`voxputd` auto-starts via D-Bus the first time you press the shortcut — no
separate systemd setup required.

### Installation

**1. Build and install the binaries** (if not done already):

```bash
cargo build --release
install -Dm755 target/release/voxput   ~/.local/bin/voxput
install -Dm755 target/release/voxputd  ~/.local/bin/voxputd
```

**2. Set your API key** so the daemon can reach Groq:

```bash
systemctl --user set-environment GROQ_API_KEY=gsk_...
```

Or add `api_key = "gsk_..."` under `[providers.groq]` in
`~/.config/voxput/config.toml`.

**3. Install the extension and D-Bus activation file**:

```bash
cd extensions/gnome
make install
```

This copies the extension to `~/.local/share/gnome-shell/extensions/` and
installs the D-Bus session service file to `~/.local/share/dbus-1/services/`,
which tells the session bus how to auto-start `voxputd`.

**4. Restart GNOME Shell and enable the extension**:

```bash
# Wayland: log out and log back in
# X11: Alt+F2 → type 'r' → Enter

gnome-extensions enable voxput@jonochang.github.com
```

Or toggle it on in the **Extensions** app.

### Usage

The default mode is **push-to-talk**: hold **Super+M** to record, release to
stop. The transcript is copied to the clipboard automatically.

- **Hold Super+M** → recording starts (icon turns red)
- **Release** → transcription runs (icon turns yellow), then transcript is
  copied to clipboard
- **Switch to target window** and press `Ctrl+V` to paste
- Click the top-bar icon to open the popup menu (shows last transcript, manual
  Start/Stop toggle)

In **toggle mode** (switchable in Settings): press once to start, press again
to stop.

### Top-bar icon states

| Icon | Colour | Meaning |
|------|--------|---------|
| Muted mic | Grey | Idle |
| Mic | Red | Recording |
| Spinner | Yellow | Transcribing |
| Warning | Red | Error |

### Settings

Open the **Extensions** app → Voxput → ⚙, or run:

```bash
gnome-extensions prefs voxput@jonochang.github.com
```

Options:
- **Push-to-talk** — hold shortcut to record, release to stop (default on);
  switch off for toggle mode (press once to start, press again to stop)
- **Shortcut** — change the keybinding (click "Change" and press the desired
  combination)
- **Show transcript notification** — toggle the GNOME notification on completion
- **Auto-paste transcript** — type the transcript into the focused window
  automatically (requires `programs.ydotool.enable = true` and the user in the
  `ydotool` group in your NixOS system config)
- **Auto-start voxputd** — start the daemon automatically when the extension
  enables (on by default; requires the D-Bus activation file to be installed)

### Uninstall

```bash
cd extensions/gnome
make uninstall
```

## NixOS / Home Manager integration

Add voxput to your flake inputs and apply the overlay, then import the Home
Manager module. The module manages the `voxputd` systemd user service, the
D-Bus activation file, and optionally the GNOME Shell extension.

### 1. Add the flake input

```nix
# flake.nix
inputs.voxput.url = "github:jonochang/voxput";
```

### 2. Apply the overlay

This makes `pkgs.voxput` and `pkgs.voxputGnomeExtension` available:

```nix
nixpkgs.overlays = [ inputs.voxput.overlays.default ];
```

### 3. Import the Home Manager module

```nix
home-manager.users.alice = {
  imports = [ inputs.voxput.homeManagerModules.default ];
};
```

### 4. Configure

Minimal (daemon only):

```nix
services.voxput.enable = true;
```

With GNOME extension:

```nix
services.voxput = {
  enable = true;
  gnome = {
    enable           = true;
    shortcut         = [ "<Super>v" ];   # default: Super+M
    showNotification = true;             # default: true
    pushToTalk       = true;             # default: true (false = toggle mode)
    autoPaste        = false;            # default: false; requires ydotool (see below)
  };
};

# Enable the extension in GNOME Shell (Home Manager 23.11+):
programs.gnome-shell.extensions = [
  { package = config.services.voxput.gnome.package; }
];
```

### API key

Do not put your API key directly in your Nix config — it will end up in
the world-readable Nix store and in your git history.

**Option A — manual config file** (simplest, no secrets manager needed):

Create `~/.config/voxput/config.toml` outside of Nix:

```toml
[providers.groq]
api_key = "gsk_..."
```

The daemon reads this file at startup. Keep the file out of version control.

**Option B — systemd user environment**:

```bash
systemctl --user set-environment GROQ_API_KEY=gsk_...
systemctl --user restart voxputd
```

Add to your shell profile or `~/.config/environment.d/groq.conf` to persist
across reboots.

**Option C — secrets manager** (`apiKeyFile`):

Pass a file whose contents are shell-style environment variable assignments:

```
GROQ_API_KEY=gsk_...
```

With **sops-nix**:

```nix
sops.secrets.groq-api-key = { sopsFile = ./secrets.yaml; };
services.voxput.apiKeyFile = config.sops.secrets.groq-api-key.path;
```

With **agenix**:

```nix
age.secrets.groq-api-key = { file = ./secrets/groq-api-key.age; };
services.voxput.apiKeyFile = config.age.secrets.groq-api-key.path;
```

### All module options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | — | Enable the voxputd daemon |
| `package` | package | `pkgs.voxput` | The voxput package |
| `apiKeyFile` | path\|null | `null` | File containing `GROQ_API_KEY=…` (runtime secret) |
| `model` | str\|null | `null` | Whisper model (default: `whisper-large-v3-turbo`) |
| `device` | str\|null | `null` | Audio input device name |
| `gnome.enable` | bool | — | Enable the GNOME Shell extension |
| `gnome.package` | package | `pkgs.voxputGnomeExtension` | Extension package |
| `gnome.shortcut` | \[str\] | `["<Super>m"]` | Toggle-recording keybinding |
| `gnome.showNotification` | bool | `true` | Notify on transcript completion |
| `gnome.pushToTalk` | bool | `true` | Hold to record / release to stop; `false` = toggle mode |
| `gnome.autoPaste` | bool | `false` | Auto-type transcript into focused window (requires ydotool) |

#### Auto-paste setup (NixOS)

`autoPaste` uses `ydotool` to inject keystrokes at the kernel level. Two
things are required in your **NixOS system** config (not Home Manager):

```nix
programs.ydotool.enable = true;
users.users.<youruser>.extraGroups = [ "ydotool" ];
```

Log out and back in after applying for the group membership to take effect.

## Roadmap

- **v0.1 (done):** CLI one-shot mode — `voxput record`
- **v0.2 (done):** Background daemon (`voxputd`) with D-Bus IPC
- **v0.3 (done):** GNOME Shell extension — top-bar indicator, configurable
  shortcut, push-to-talk and toggle recording modes, auto-paste via ydotool,
  NixOS/Home Manager module
- **v0.4:** Additional transcription providers (OpenAI, local whisper.cpp),
  language selection in the extension UI
