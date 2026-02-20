# Home Manager Module

## Minimal (daemon only)

```nix
services.voxput.enable = true;
```

This installs the `voxput` and `voxputd` binaries, sets up the systemd user
service, and installs the D-Bus activation file.

## With GNOME extension

```nix
services.voxput = {
  enable = true;
  gnome = {
    enable           = true;
    shortcut         = [ "<Super>v" ];   # default: Super+M
    showNotification = true;             # default: true
    pushToTalk       = true;             # default: true (false = toggle mode)
    autoPaste        = false;            # default: false; requires ydotool
  };
};

# Enable the extension in GNOME Shell (Home Manager 23.11+):
programs.gnome-shell.extensions = [
  { package = config.services.voxput.gnome.package; }
];
```

## Auto-paste setup

If you enable `autoPaste`, two things are required in your **NixOS system**
config (not Home Manager):

```nix
programs.ydotool.enable = true;
users.users.<youruser>.extraGroups = [ "ydotool" ];
```

Log out and back in after applying for the group membership to take effect.

See [Auto-Paste](../gnome-extension/auto-paste.md) for details.

## API key

See [API Key Management](../daemon/api-key.md) for all available methods
(config file, systemd environment, sops-nix, agenix).
