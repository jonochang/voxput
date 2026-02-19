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
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "clippy" "rustfmt" "rust-src" ];
        };

        voxputPkg = pkgs.callPackage ./package.nix { };
      in
      {
        packages.voxput = voxputPkg;
        packages.default = voxputPkg;

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
    );
}
