# Installation

## Requirements

- Rust toolchain
- A [Groq API key](https://console.groq.com/) (free tier available)
- A microphone accessible via ALSA/PipeWire

## From source

```bash
git clone https://github.com/jonochang/voxput
cd voxput
cargo build --release

# Install both binaries to ~/.local/bin (or any directory on your $PATH)
install -Dm755 target/release/voxput   ~/.local/bin/voxput
install -Dm755 target/release/voxputd  ~/.local/bin/voxputd
```

## NixOS / Home Manager

If you use NixOS with Home Manager, see the
[NixOS / Home Manager](./nixos/README.md) section for declarative installation
via the voxput flake.
