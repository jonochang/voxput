# Voxput — Architecture & Design

## Current Architecture (v0.3)

The current implementation consists of three components:

```
voxput-core (Rust library)
  ├── Audio capture (cpal)
  ├── Groq Whisper provider
  ├── Output (stdout / clipboard)
  ├── Config (TOML + env vars)
  └── State machine (Idle/Recording/Transcribing/Error)

voxputd (Rust daemon)
  ├── D-Bus session service (com.github.jonochang.Voxput1)
  ├── StartRecording / StopRecording / Toggle / GetStatus
  ├── StateChanged signal
  └── Clipboard output on transcription complete

voxput CLI
  ├── One-shot: voxput record (direct mic → API → stdout)
  └── Daemon control: voxput start/stop/toggle/status (D-Bus client)

GNOME Shell extension (JavaScript, GNOME 45+)
  ├── Top-bar indicator (idle/recording/transcribing/error)
  ├── Push-to-talk (Main.pushModal keyboard grab + key-release detection)
  ├── Toggle mode (press once to start, again to stop)
  ├── Auto-paste via ydotool (kernel uinput, bypasses Wayland)
  ├── Clipboard fallback (always set)
  ├── GSettings preferences (shortcut, mode, auto-paste, notifications)
  └── D-Bus activation (auto-starts voxputd)
```

### IPC

D-Bus on the session bus. The GNOME extension communicates directly with
`voxputd` — no bridge or helper binary needed.

### Input injection (auto-paste)

Lessons learned on GNOME Wayland:

- **wtype**: Uses `zwlr_virtual_keyboard_manager_v1`, a wlroots-only protocol.
  Silently fails on GNOME Mutter.
- **Clutter VirtualInputDevice**: Routes through gnome-shell's internal event
  pipeline. Hundreds of rapid `notify_keyval` calls destabilise the shell and
  crash it back to GDM.
- **ydotool** (current): Injects at the kernel uinput level, completely
  bypassing the Wayland compositor. Requires `ydotoold` daemon and the user
  in the `ydotool` group.

### Push-to-talk key-release detection

On Wayland, key-release events from WM keybindings are delivered to the
focused application's surface, not to gnome-shell's Clutter stage.
`global.stage.connect('key-release-event')` does not fire when an app has
focus. The extension uses `Main.pushModal()` to temporarily grab keyboard
input to the shell during recording, ensuring the release event is delivered
reliably.

### Technologies (current)

| Component | Technology |
|-----------|-----------|
| Language | Rust |
| Async | tokio |
| HTTP | reqwest |
| Audio | cpal (ALSA/PipeWire) |
| IPC | D-Bus (zbus) |
| CLI | clap |
| Extension | GJS (GNOME Shell 45+ ESM modules) |
| Auto-paste | ydotool (kernel uinput) |
| Packaging | Nix flake, Home Manager module |

---

## Future Vision

The sections below describe aspirational architecture for expanding beyond
GNOME/Linux. None of this is implemented yet.

### Cross-platform layers

Split voxput into a portable Rust core and thin platform front-ends. GNOME is
just one front-end; later add macOS, Windows, KDE, etc. without rewriting
providers/audio/transcription logic.

```
1. voxput-core (portable Rust library)
   - Audio capture abstraction (platform backends)
   - Provider abstraction (Groq now; local Whisper/Ollama later)
   - Post-processing pipeline (optional)
   - Output abstraction (stdout/clipboard/type-at-cursor)
   - State machine (Idle/Recording/Transcribing/Error)

2. voxputd (local service/daemon, cross-platform)
   - Hosts the state machine + orchestrates audio → provider → output
   - Exposes an IPC API to front-ends
   - Emits state updates for indicators/UX

3. Front-ends (platform-specific)
   - GNOME Shell extension (implemented)
   - KDE / Plasma widget or KGlobalAccel integration
   - macOS menu bar app
   - Windows tray app

4. voxput CLI
   - Scriptable entrypoint, prints to stdout
   - Can control voxputd via IPC or run one-shot mode directly
```

### Additional IPC options

The current D-Bus approach works well for Linux desktops. For cross-platform
support, a gRPC-over-Unix-socket (or named pipe on Windows) API could serve
as a universal control interface, with platform-specific bridges where needed.

### Additional providers

- OpenAI-compatible endpoint (same multipart API as Groq)
- Local whisper.cpp or faster-whisper
- Hybrid: local STT + optional LLM cleanup via Ollama

### Additional output backends

| Platform | Approach |
|----------|----------|
| Linux Wayland | ydotool (implemented) |
| Linux X11 | xdotool |
| macOS | Accessibility APIs / CGEvent |
| Windows | SendInput / UI Automation |

Clipboard fallback should always be available regardless of platform.

### Platform packaging

- Linux: systemd user service + GNOME extension (implemented via Nix)
- macOS: launchd agent + menu bar app
- Windows: startup task + tray app
