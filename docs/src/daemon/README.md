# Daemon Overview

`voxputd` is a background daemon that exposes a D-Bus service on the session
bus. It holds a persistent microphone stream and lets you start/stop recording
from any other program without keeping a terminal open.

This is the backend for the GNOME Shell extension and for hotkey-based
recording workflows.

## D-Bus interface

- **Bus name:** `com.github.jonochang.Voxput`
- **Object path:** `/com/github/jonochang/Voxput`
- **Interface:** `com.github.jonochang.Voxput1`

### Methods

| Method | Description |
|--------|-------------|
| `StartRecording` | Begin capturing audio |
| `StopRecording` | Stop and transcribe |
| `Toggle` | Start if idle, stop if recording |
| `GetStatus` | Returns `(state, transcript, error)` |

### Signals

| Signal | Arguments | Description |
|--------|-----------|-------------|
| `StateChanged` | `(state, transcript)` | Emitted on every state transition |

Watch signals with:

```bash
dbus-monitor "type='signal',interface='com.github.jonochang.Voxput1'"
```
