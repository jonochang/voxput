# Auto-Paste

When auto-paste is enabled, the transcript is typed into the focused window
automatically after transcription completes. The transcript is also always
copied to the clipboard as a fallback.

## How it works

Auto-paste uses [ydotool](https://github.com/RealOriginal/ydotool) to inject
keystrokes at the Linux kernel's uinput level. This bypasses the Wayland
compositor entirely, which is the only reliable approach on GNOME Wayland.

### Why not other tools?

- **wtype** uses `zwlr_virtual_keyboard_manager_v1`, a wlroots-only Wayland
  protocol. GNOME Mutter does not implement it — wtype silently fails.
- **Clutter VirtualInputDevice** (`notify_keyval`) routes through gnome-shell's
  internal event pipeline. Sending hundreds of rapid key events destabilises
  the shell and can crash it back to GDM.
- **ydotool** operates at the kernel level via `/dev/uinput`, completely
  independent of the Wayland compositor.

## Setup

### 1. Enable ydotool daemon (NixOS)

In your **NixOS system** configuration (not Home Manager):

```nix
programs.ydotool.enable = true;
```

### 2. Add your user to the ydotool group

```nix
users.users.<youruser>.extraGroups = [ "ydotool" ];
```

### 3. Rebuild and log out/in

```bash
sudo nixos-rebuild switch --flake .
```

Log out and back in for the group membership to take effect.

### 4. Enable auto-paste in the extension

In the extension settings, turn on **Auto-paste transcript**.

Or via Home Manager:

```nix
services.voxput.gnome.autoPaste = true;
```

## Verifying ydotool

```bash
# Check the daemon is running
systemctl status ydotoold

# Check socket permissions
ls -la /run/ydotoold/

# Test injection
ydotool type -- "hello"
```

If you see "Permission denied" on the socket, your user is not in the
`ydotool` group.

## Clipboard fallback

The transcript is always copied to the clipboard regardless of whether
ydotool is available. If ydotool fails or is not installed, you can paste
manually with `Ctrl+V`.
