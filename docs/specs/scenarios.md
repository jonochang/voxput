# Scenarios

Behavior-focused scenarios in Given/When/Then form.

## CLI

### 1. Record to stdout (default)
Given a user has a working microphone and `GROQ_API_KEY` is set
When they run `voxput record`
Then the tool waits until a key is pressed to stop recording and prints the transcript to stdout

### 2. Record with custom duration
Given a user has a working microphone and `GROQ_API_KEY` is set
When they run `voxput record --duration 10`
Then the tool records audio for 10 seconds and prints the transcript to stdout

### 3. Record with language hint
Given a user speaks French and has `GROQ_API_KEY` set
When they run `voxput record --language fr`
Then the transcript is returned in French when possible

### 4. Record to clipboard
Given a user has a working clipboard in their desktop session and `GROQ_API_KEY` is set
When they run `voxput record --output clipboard`
Then the transcript is copied to the clipboard and not printed to stdout

### 5. Record to stdout and clipboard
Given a user has a working clipboard and `GROQ_API_KEY` is set
When they run `voxput record --output both`
Then the transcript is printed to stdout and copied to the clipboard

### 6. Record with device selection
Given a user has multiple audio input devices
When they run `voxput record --device "USB Audio"`
Then audio is captured from the named device

### 7. List devices (plain text)
Given a user has one or more audio input devices
When they run `voxput devices`
Then the tool prints a list of device names and marks the default device

### 8. List devices (JSON)
Given a user has one or more audio input devices
When they run `voxput devices --json`
Then the tool prints a JSON array of device objects with `name` and `is_default`

### 9. Missing API key
Given `GROQ_API_KEY` is not set and no API key is in config
When the user runs `voxput record`
Then the command fails with a clear message instructing how to set the API key

### 10. Provider error
Given the Groq API is unreachable or returns an error
When the user runs `voxput record`
Then the command fails with a user-friendly error message that includes the HTTP status

## Daemon

### 11. Daemon control via CLI
Given the daemon is running
When the user runs `voxput start`, `voxput stop`, or `voxput toggle`
Then the daemon starts/stops recording or toggles accordingly, and `voxput status` reflects the current state

### 12. Daemon auto-start via D-Bus activation
Given the daemon is not running but the D-Bus activation file is installed
When the GNOME extension calls a method on `com.github.jonochang.Voxput1`
Then the daemon starts automatically via D-Bus activation

## GNOME Extension — Push-to-talk

### 13. Push-to-talk success
Given the daemon is running and the GNOME extension is installed with push-to-talk mode enabled
When the user holds the configured shortcut and speaks
Then the indicator shows Recording while the key is held, Transcribing after release, and the transcript is copied to the clipboard

### 14. Push-to-talk key-release detection
Given the user is in a focused application window on Wayland
When the user holds and then releases the push-to-talk shortcut
Then the extension detects the key release via `Main.pushModal` keyboard grab and stops recording

### 15. Push-to-talk network failure
Given the daemon is running but the network is down
When the user records via push-to-talk
Then the indicator shows Error and the failure is visible via a notification

## GNOME Extension — Toggle mode

### 16. Toggle mode success
Given the extension is installed with push-to-talk mode disabled (toggle mode)
When the user presses the shortcut once
Then recording starts and the indicator shows Recording
When the user presses the shortcut again
Then recording stops, transcription runs, and the transcript is copied to the clipboard

### 17. Toggle mode ignores press during transcription
Given the extension is in toggle mode and transcription is in progress
When the user presses the shortcut
Then nothing happens — the shortcut is ignored until the state returns to idle

## GNOME Extension — Auto-paste

### 18. Auto-paste with ydotool
Given auto-paste is enabled, `ydotoold` is running, and the user is in the `ydotool` group
When transcription completes
Then the transcript is typed into the focused window via `ydotool type`

### 19. Auto-paste clipboard fallback
Given auto-paste is enabled but `ydotool` is not installed or the daemon is not running
When transcription completes
Then the transcript is still copied to the clipboard (fallback) and no crash occurs

### 20. Auto-paste permission denied
Given `ydotoold` is running but the user is not in the `ydotool` group
When the extension tries to auto-paste
Then `ydotool type` fails with a permission error, the transcript is still on the clipboard, and the error is logged

## GNOME Extension — Recovery

### 21. Shortcut works after error state
Given the daemon entered an error state from a previous failed recording
When the user presses the push-to-talk or toggle shortcut
Then a new recording starts normally (the error state does not block new recordings)
