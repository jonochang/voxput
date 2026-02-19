{
  description = "voxput - voice-to-text dictation tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    let
      # ------------------------------------------------------------------
      # System-agnostic outputs
      # ------------------------------------------------------------------

      # Overlay: adds pkgs.voxput and pkgs.voxputGnomeExtension.
      # Apply this in your NixOS / Home Manager config:
      #   nixpkgs.overlays = [ inputs.voxput.overlays.default ];
      overlays.default = final: _prev: {
        # Pass src = self so package.nix builds from the flake source
        # directly, without needing a git tag.
        voxput = final.callPackage ./package.nix { src = self; };
        voxputGnomeExtension = final.callPackage ./nix/gnome-extension.nix { };
      };

      # Home Manager module.  Import in your HM config:
      #   imports = [ inputs.voxput.homeManagerModules.default ];
      homeManagerModules.default = import ./nix/home-manager-module.nix;

    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "clippy" "rustfmt" "rust-src" ];
        };
      in
      {
        # Packages
        packages.voxput            = pkgs.voxput;
        packages.gnome-extension   = pkgs.voxputGnomeExtension;
        packages.default           = pkgs.voxput;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain

            # Native build dependencies (cpal → ALSA on Linux)
            pkgs.pkg-config
            pkgs.alsa-lib

            # Cargo dev tools
            pkgs.cargo-nextest
            pkgs.cargo-insta

            # Test fixture generation (espeak-ng voice WAV via ffmpeg)
            pkgs.espeak-ng
            pkgs.ffmpeg
          ];
        };
      }
    ) // {
      inherit overlays homeManagerModules;
    };
}
