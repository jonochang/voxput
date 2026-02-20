# Settings

Open the extension settings from the **Extensions** app (gear icon on
Voxput), or via the command line:

```bash
gnome-extensions prefs voxput@jonochang.github.com
```

## Available settings

### Push-to-talk

Hold the shortcut to record, release to stop. When off, the shortcut toggles
recording (press once to start, press again to stop).

**Default:** on

### Shortcut

The keyboard shortcut for recording. Click **Change** and press the desired
key combination.

**Default:** Super+M

### Show transcript notification

Display a GNOME notification when transcription completes.

**Default:** on

### Auto-paste transcript

Type the transcript into the focused window automatically after transcription.
Requires ydotool — see [Auto-Paste](./auto-paste.md).

**Default:** off

### Auto-start voxputd

Start the `voxputd` daemon automatically when the extension enables. Requires
the D-Bus activation file to be installed.

**Default:** on

## GSettings keys

All settings are stored under `org.gnome.shell.extensions.voxput`:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `toggle-recording` | `as` | `['<Super>m']` | Keyboard shortcut |
| `push-to-talk` | `b` | `true` | Push-to-talk mode |
| `show-transcript-notification` | `b` | `true` | Show notifications |
| `auto-paste` | `b` | `false` | Auto-paste via ydotool |
| `daemon-auto-start` | `b` | `true` | Auto-start voxputd |

You can read/write these directly with `dconf`:

```bash
dconf read /org/gnome/shell/extensions/voxput/push-to-talk
dconf write /org/gnome/shell/extensions/voxput/push-to-talk false
```
