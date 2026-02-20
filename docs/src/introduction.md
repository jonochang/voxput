# Introduction

Voxput is a voice-to-text dictation tool for Linux. Speak into your mic, get
text out.

It is powered by the [Groq Whisper API](https://console.groq.com/docs/speech-text)
for fast transcription and designed to feel native on GNOME Wayland.

## Components

- **`voxput`** — CLI tool for one-shot recording and daemon control
- **`voxputd`** — background daemon exposing a D-Bus service on the session bus
- **GNOME Shell extension** — top-bar indicator with push-to-talk shortcut

## Key features

- Push-to-talk and toggle recording modes via a configurable keyboard shortcut
- Top-bar indicator showing idle, recording, transcribing, and error states
- Auto-paste transcripts into the focused window via ydotool
- Clipboard fallback (always available)
- NixOS / Home Manager module for declarative configuration
- D-Bus activation (daemon auto-starts on first use)
