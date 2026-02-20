# Quick Start

## 1. Set your API key

```bash
export GROQ_API_KEY=gsk_...
```

## 2. Record and transcribe (CLI)

```bash
voxput record
```

Speak into your mic, press any key to stop. The transcript prints to stdout.

## 3. Start the daemon

```bash
voxputd
```

Or via systemd:

```bash
cp contrib/voxputd.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now voxputd
```

## 4. Install the GNOME extension

```bash
cd extensions/gnome
make install
```

Log out and back in, then enable the extension:

```bash
gnome-extensions enable voxput@jonochang.github.com
```

Hold **Super+M** to record (push-to-talk). Release to transcribe. The
transcript is copied to the clipboard automatically.

See the [GNOME Extension](./gnome-extension/README.md) section for full
details.
