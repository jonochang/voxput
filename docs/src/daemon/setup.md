# Daemon Setup

## Install and start

```bash
# Copy the systemd service unit
cp contrib/voxputd.service ~/.config/systemd/user/

# Tell systemd to pick it up, then enable and start
systemctl --user daemon-reload
systemctl --user enable --now voxputd

# Confirm it is running
systemctl --user status voxputd
```

To run the daemon manually (foreground, logs to stderr):

```bash
GROQ_API_KEY=gsk_... voxputd
```

## Verify the connection

```bash
voxput status
# state:      idle
```

## Check daemon logs

```bash
journalctl --user -u voxputd -f
```

Set `RUST_LOG=debug` for verbose output:

```bash
systemctl --user set-environment RUST_LOG=debug
systemctl --user restart voxputd
```

## NixOS

On NixOS with Home Manager, the daemon is managed automatically when you
enable `services.voxput`. See [Home Manager Module](../nixos/home-manager.md).
