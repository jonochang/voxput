# CLI Overview

The `voxput` CLI has two modes of operation:

1. **One-shot mode** — record and transcribe directly (`voxput record`)
2. **Daemon control** — send commands to `voxputd` (`voxput start`, `stop`,
   `toggle`, `status`)

## Commands

| Command | Description |
|---------|-------------|
| `record` | Record audio and transcribe (one-shot, no daemon needed) |
| `devices` | List available audio input devices |
| `start` | Tell the daemon to start recording |
| `stop` | Tell the daemon to stop recording and transcribe |
| `toggle` | Start if idle, stop if recording |
| `status` | Show daemon state (idle / recording / transcribing / error) |

Status messages go to stderr so stdout is clean for piping:

```bash
voxput record >> notes.txt
voxput record | wl-copy     # Wayland clipboard
voxput record | xclip       # X11 clipboard
```
