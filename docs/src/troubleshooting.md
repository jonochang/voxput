# Troubleshooting

## Daemon won't start

**Symptom:** `voxput status` shows "Could not connect to voxputd"

- Check if the daemon is running: `systemctl --user status voxputd`
- Check logs: `journalctl --user -u voxputd -f`
- Verify the API key is set: `echo $GROQ_API_KEY` or check
  `~/.config/voxput/config.toml`

## Shortcut does nothing

**Symptom:** Pressing Super+M has no effect

- Verify the extension is enabled: `gnome-extensions list --enabled`
- Check the shortcut is set: `dconf read /org/gnome/shell/extensions/voxput/toggle-recording`
- Check gnome-shell logs for errors: `journalctl -f /usr/bin/gnome-shell`
- If the previous recording errored, the shortcut should still work (error
  state does not block new recordings). If it doesn't, try disabling and
  re-enabling the extension.

## Push-to-talk doesn't stop on key release

**Symptom:** Recording starts when you press the shortcut but doesn't stop
when you release

- This can happen if `Main.pushModal()` fails to grab the keyboard. Press
  the shortcut again to stop recording manually.
- Check gnome-shell logs: `journalctl -f /usr/bin/gnome-shell` for
  "pushModal failed" messages.

## Auto-paste doesn't work

**Symptom:** Transcription completes but text is not typed into the window

1. **Check ydotool is installed:** `which ydotool`
2. **Check ydotoold is running:** `systemctl status ydotoold`
3. **Check socket permissions:** `ls -la /run/ydotoold/`
4. **Check your groups:** `groups` — you need to be in the `ydotool` group
5. **Test manually:** `ydotool type -- "hello"`

If you see "Permission denied", add yourself to the ydotool group:

```nix
users.users.<youruser>.extraGroups = [ "ydotool" ];
```

Rebuild and log out/in for the group change to take effect.

The transcript is always on the clipboard as a fallback — paste with `Ctrl+V`.

## Extension settings don't appear

**Symptom:** The settings panel is empty or the extension shows an error

- Verify schemas are compiled: check for `gschemas.compiled` in the
  extension's `schemas/` directory
- On NixOS, the Home Manager module handles schema compilation automatically
- For manual installs, run `glib-compile-schemas` on the extension's
  `schemas/` directory

## Extension changes don't take effect

On Wayland, GNOME Shell caches ES modules. You must **log out and back in**
for extension JavaScript changes to reload. `gnome-extensions disable/enable`
is not sufficient.
