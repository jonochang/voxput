# Architecture

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

## Component overview

```
┌─────────────────────────────────────────────────┐
│                  GNOME Shell                     │
│  ┌──────────────────────────────────────────┐   │
│  │  voxput extension (JavaScript, ESM)      │   │
│  │  - Top-bar indicator                     │   │
│  │  - Push-to-talk (Main.pushModal grab)    │   │
│  │  - Toggle mode                           │   │
│  │  - Auto-paste (ydotool subprocess)       │   │
│  │  - GSettings preferences                │   │
│  └──────────────┬───────────────────────────┘   │
│                 │ D-Bus (session bus)            │
│  ┌──────────────▼───────────────────────────┐   │
│  │  voxputd (Rust daemon)                   │   │
│  │  - State machine (idle/recording/...)    │   │
│  │  - Audio capture (cpal)                  │   │
│  │  - Groq Whisper API client               │   │
│  │  - Clipboard output                      │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  voxput CLI (Rust)                               │
│  - One-shot: record → transcribe → stdout        │
│  - Daemon control: start/stop/toggle/status      │
│  - Communicates with voxputd via D-Bus           │
└──────────────────────────────────────────────────┘
```

## Crate structure

```
crates/
├── voxput-core/    # Portable library: audio, provider, output, config, state
├── voxput-cli/     # CLI binary (depends on voxput-core)
└── voxputd/        # Daemon binary (depends on voxput-core, zbus)
```

`voxput-core` has no dependency on D-Bus or clap, keeping it portable for
potential future front-ends.

## State machine

The daemon tracks four states:

```
         StartRecording
  idle ──────────────────► recording
   ▲                          │
   │  TranscriptionComplete   │ StopRecording
   │                          ▼
   └──────────────────── transcribing
                              │
              TranscriptionFailed
                              ▼
                           error
                              │
                  StartRecording (retry)
                              ▼
                          recording
```

The extension allows starting a new recording from both `idle` and `error`
states. Only `transcribing` blocks a new recording.

## Input injection on GNOME Wayland

See [Auto-Paste](./gnome-extension/auto-paste.md) for the full rationale on
why ydotool is the only reliable approach on GNOME Wayland.
