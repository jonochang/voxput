# Flake Setup

## 1. Add the flake input

```nix
# flake.nix
inputs.voxput.url = "github:jonochang/voxput";
```

## 2. Apply the overlay

This makes `pkgs.voxput` and `pkgs.voxputGnomeExtension` available:

```nix
nixpkgs.overlays = [ inputs.voxput.overlays.default ];
```

## 3. Import the Home Manager module

```nix
home-manager.users.alice = {
  imports = [ inputs.voxput.homeManagerModules.default ];
};
```

## Flake outputs

The voxput flake provides:

| Output | Description |
|--------|-------------|
| `overlays.default` | Adds `pkgs.voxput` and `pkgs.voxputGnomeExtension` |
| `homeManagerModules.default` | Home Manager module |
| `packages.<system>.voxput` | The voxput/voxputd binaries |
| `packages.<system>.gnome-extension` | The GNOME Shell extension |
| `devShells.<system>.default` | Development shell with Rust toolchain |
