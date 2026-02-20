# NixOS / Home Manager

Voxput provides a Nix flake with an overlay and a Home Manager module. The
module manages:

- The `voxputd` systemd user service
- D-Bus activation file
- GNOME Shell extension (optional)
- All GSettings / dconf configuration

## Quick setup

```nix
# flake.nix
inputs.voxput.url = "github:jonochang/voxput";

# In your NixOS/HM config:
nixpkgs.overlays = [ inputs.voxput.overlays.default ];

home-manager.users.alice = {
  imports = [ inputs.voxput.homeManagerModules.default ];

  services.voxput = {
    enable = true;
    gnome.enable = true;
  };
};
```

See the following pages for details:

- [Flake Setup](./flake-setup.md) — adding the flake input and overlay
- [Home Manager Module](./home-manager.md) — configuring the module
- [Module Options Reference](./module-options.md) — full options table
