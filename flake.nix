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
      #
      # Builds from the flake source directly (no git tag required).
      # package.nix is kept separately for eventual nixpkgs submission.
      overlays.default = final: _prev: {
        voxput = final.rustPlatform.buildRustPackage {
          pname = "voxput";
          version = "0.3.0";

          src = self;

          # Reads Cargo.lock from source — no cargoHash to maintain.
          cargoLock.lockFile = self + "/Cargo.lock";

          nativeBuildInputs = [ final.pkg-config ];
          buildInputs = [ final.alsa-lib ];

          checkFlags = [ "--skip=groq_integration" ];

          meta = with final.lib; {
            description = "Voice-to-text dictation tool powered by Groq Whisper";
            homepage = "https://github.com/jonochang/voxput";
            license = licenses.mit;
            mainProgram = "voxput";
            platforms = platforms.linux;
          };
        };
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
