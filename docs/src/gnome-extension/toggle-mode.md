# Toggle Mode

Toggle mode is an alternative to push-to-talk. Press the shortcut once to
start recording, press again to stop.

## How it works

1. **Press Super+M** — recording starts (icon turns red)
2. Speak
3. **Press Super+M** again — recording stops
4. Transcription runs (icon turns yellow)
5. Transcript is copied to the clipboard
6. Icon returns to grey (idle)

## When to use toggle mode

Toggle mode is useful when:

- You want to record longer dictations without holding a key
- Your keyboard or shortcut combination makes holding uncomfortable
- You prefer an explicit start/stop workflow

## Enabling toggle mode

Open the extension settings and turn **off** the **Push-to-talk** switch:

```bash
gnome-extensions prefs voxput@jonochang.github.com
```

Or via dconf:

```bash
dconf write /org/gnome/shell/extensions/voxput/push-to-talk false
```

With Home Manager:

```nix
services.voxput.gnome.pushToTalk = false;
```

## Behavior during transcription

If you press the shortcut while transcription is in progress, the keypress
is ignored. A new recording can only start after the previous transcription
completes (or errors).
