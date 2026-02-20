# GNOME Extension Overview

The extension lives in `extensions/gnome/`. It adds a microphone indicator to
the top bar and a configurable keyboard shortcut for hands-free dictation.
`voxputd` auto-starts via D-Bus the first time you press the shortcut.

## Installation

**1. Build and install the binaries** (if not done already):

```bash
cargo build --release
install -Dm755 target/release/voxput   ~/.local/bin/voxput
install -Dm755 target/release/voxputd  ~/.local/bin/voxputd
```

**2. Set your API key** so the daemon can reach Groq:

```bash
systemctl --user set-environment GROQ_API_KEY=gsk_...
```

Or add `api_key = "gsk_..."` under `[providers.groq]` in
`~/.config/voxput/config.toml`.

**3. Install the extension and D-Bus activation file**:

```bash
cd extensions/gnome
make install
```

This copies the extension to `~/.local/share/gnome-shell/extensions/` and
installs the D-Bus session service file to `~/.local/share/dbus-1/services/`.

**4. Restart GNOME Shell and enable the extension**:

```bash
# Wayland: log out and log back in
# X11: Alt+F2 → type 'r' → Enter

gnome-extensions enable voxput@jonochang.github.com
```

Or toggle it on in the **Extensions** app.

## Top-bar icon states

| Icon | Colour | Meaning |
|------|--------|---------|
| Muted mic | Grey | Idle |
| Mic | Red | Recording |
| Spinner | Yellow | Transcribing |
| Warning | Red | Error |

## Uninstall

```bash
cd extensions/gnome
make uninstall
```
