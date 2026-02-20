# Push-to-Talk

Push-to-talk is the default recording mode. Hold the shortcut key to record,
release to stop and transcribe.

## How it works

1. **Hold Super+M** (or your configured shortcut)
2. The top-bar icon turns red (recording)
3. Speak
4. **Release the key**
5. The icon turns yellow (transcribing)
6. The transcript is copied to the clipboard
7. The icon returns to grey (idle)

## Key-release detection on Wayland

On Wayland, key-release events from WM keybindings are delivered to the
focused application's surface, not back to gnome-shell. The extension uses
`Main.pushModal()` to temporarily grab keyboard input to the shell for the
duration of the recording. This ensures the key release is detected reliably.

While recording, keyboard input to application windows is paused. This is
expected — you are speaking, not typing. Input returns to normal as soon as
the key is released.

## Fallback

If push-to-talk key-release detection fails (e.g., `pushModal` could not
acquire the grab), pressing the shortcut again will stop recording via the
state-based fallback.

## Switching modes

To switch to toggle mode, open the extension settings and turn off the
**Push-to-talk** switch. See [Toggle Mode](./toggle-mode.md).
