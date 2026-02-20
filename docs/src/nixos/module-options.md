# Module Options Reference

## All options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | -- | Enable the voxputd daemon |
| `package` | package | `pkgs.voxput` | The voxput package |
| `apiKeyFile` | path\|null | `null` | File containing `GROQ_API_KEY=...` (runtime secret) |
| `model` | str\|null | `null` | Whisper model (default: `whisper-large-v3-turbo`) |
| `device` | str\|null | `null` | Audio input device name |
| `gnome.enable` | bool | -- | Enable the GNOME Shell extension |
| `gnome.package` | package | `pkgs.voxputGnomeExtension` | Extension package |
| `gnome.shortcut` | \[str\] | `["<Super>m"]` | Toggle-recording keybinding |
| `gnome.showNotification` | bool | `true` | Notify on transcript completion |
| `gnome.pushToTalk` | bool | `true` | Hold to record / release to stop; `false` = toggle mode |
| `gnome.autoPaste` | bool | `false` | Auto-type transcript into focused window (requires ydotool) |

## Example: full configuration

```nix
services.voxput = {
  enable = true;
  apiKeyFile = config.sops.secrets.groq-api-key.path;
  model = "whisper-large-v3-turbo";

  gnome = {
    enable           = true;
    shortcut         = [ "<Super>m" ];
    showNotification = true;
    pushToTalk       = true;
    autoPaste        = true;
  };
};
```
