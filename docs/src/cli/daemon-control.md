# Daemon Control

These commands communicate with `voxputd` over D-Bus. The daemon must be
running (or D-Bus activation must be configured).

## Commands

### start

```bash
voxput start
```

Begin recording. The daemon captures audio from the microphone.

### stop

```bash
voxput stop
```

Stop recording and start transcription. When transcription completes, the
result is copied to the clipboard.

### toggle

```bash
voxput toggle
```

Start recording if idle, stop if recording. Convenient for hotkey bindings.

### status

```bash
voxput status
voxput status --json
```

Show the current daemon state: `idle`, `recording`, `transcribing`, or `error`.

## Connection errors

If the daemon is not running:

```
Error: Could not connect to voxputd
Hint: start the daemon with `voxputd` or `systemctl --user start voxputd`
```

## Hotkey bindings (non-GNOME)

If you are not using the GNOME Shell extension, any tool that can run a shell
command on a key event works:

```bash
# sxhkd (~/.config/sxhkd/sxhkdrc)
super + m
    voxput toggle

# xbindkeys (~/.xbindkeysrc)
"voxput toggle"
  Mod4 + m
```

The GNOME extension handles this natively — see
[GNOME Extension](../gnome-extension/README.md).
