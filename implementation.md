# Voxput — Implementation Plan (Historical)

> **Note:** This document was written during initial development (v0.1 MVP)
> and is preserved as a historical reference. The MVP phases (1–7) have been
> completed. The post-MVP roadmap (phases 8–13) was partially followed — see
> inline annotations for what changed during implementation.

## 1. MVP Definition

The MVP is a CLI-only, one-shot dictation tool. No daemon, no GNOME extension.

**What it does:**
- `voxput record` — captures audio from the default mic until any key is pressed, sends to Groq Whisper API, prints transcription to stdout
- `voxput record --duration 5` — timed recording; stops after 5 seconds OR on keypress, whichever comes first
- `voxput record --output clipboard` — copy result to clipboard
- `voxput record --output both` — stdout + clipboard
- `voxput devices` — list available audio input devices

**Not in MVP:**
- No daemon (`voxputd`), no IPC, no gRPC
- No GNOME Shell extension
- No push-to-talk hotkey
- No type-at-cursor
- No streaming transcription
- No local whisper.cpp provider

**End-to-end flow:**
```
voxput record
  -> enable terminal raw mode (crossterm)
  -> capture audio from default mic via cpal (16kHz mono)
  -> keypress listener thread sets stop flag on any keypress
  -> recording loop polls stop flag (and optional duration limit)
  -> encode captured samples to WAV in memory (16-bit PCM)
  -> POST WAV as multipart/form-data to Groq Whisper API
  -> parse JSON response
  -> restore terminal, write transcript text to stdout (or clipboard)
```

**Stop behaviour:**
- Default (`voxput record`): records until any key is pressed (no time limit)
- `--duration N`: records until keypress OR N seconds, whichever comes first
- The stop flag is an `Arc<AtomicBool>` shared between the keypress thread and the recording loop; either side can set it

---

## 2. Workspace Structure

Two-crate Cargo workspace. `voxput-core` is the portable library; `voxput-cli` is the thin binary. This matches the layered architecture from `design.md` and lets the future daemon depend on `voxput-core` without pulling in clap.

```
voxput/
  Cargo.toml                      # workspace root
  Cargo.lock
  README.md
  LICENSE
  .gitignore
  docs/
    specs/
      background.md
      design.md
    implementation.md              # this file
  crates/
    voxput-core/
      Cargo.toml
      src/
        lib.rs
        errors.rs                  # VoxputError, Result alias
        state.rs                   # DictationStateMachine
        audio/
          mod.rs                   # AudioBackend trait, AudioData, DeviceInfo
          cpal_backend.rs          # CpalBackend: AudioBackend impl
          wav.rs                   # encode_wav()
        provider/
          mod.rs                   # TranscriptionProvider trait, Transcript, TranscribeOptions
          groq.rs                  # GroqProvider impl
        output/
          mod.rs                   # OutputSink trait, OutputTarget enum, create_sink()
          stdout.rs                # StdoutSink
          clipboard.rs             # ClipboardSink
        config/
          mod.rs                   # load_config(), ResolvedConfig
          schema.rs                # FileConfig (TOML-deserializable)
    voxput-cli/
      Cargo.toml
      src/
        main.rs                    # tokio main, tracing init, clap dispatch
        cli/
          mod.rs                   # Cli, Commands, dispatch()
          record.rs                # RecordArgs, run()
          devices.rs               # DevicesArgs, run()
  tests/
    integration.rs
    integration/
      record_test.rs
      devices_test.rs
```

---

## 3. MVP Phases

### Phase 1 — Scaffolding

**Goal:** Workspace, crates, all dependencies declared, skeleton compiles with `cargo build`.

**`Cargo.toml` (workspace root):**
```toml
[workspace]
resolver = "2"
members = ["crates/voxput-core", "crates/voxput-cli"]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
# Async
tokio = { version = "1", features = ["full"] }

# HTTP
reqwest = { version = "0.12", features = ["multipart", "json"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Audio
cpal = "0.15"
hound = "3"

# Clipboard
arboard = "3"

# Error handling
thiserror = "2"
miette = { version = "7", features = ["fancy"] }
async-trait = "0.1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Config
dirs = "5"

# Dev
insta = { version = "1", features = ["json"] }
assert_cmd = "2"
predicates = "3"
tempfile = "3"
mockito = "1"
proptest = "1"
```

**`crates/voxput-core/Cargo.toml`:**
```toml
[package]
name = "voxput-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
tokio = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
cpal = { workspace = true }
hound = { workspace = true }
arboard = { workspace = true }
thiserror = { workspace = true }
miette = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
dirs = { workspace = true }

[dev-dependencies]
insta = { workspace = true }
tempfile = { workspace = true }
mockito = { workspace = true }
proptest = { workspace = true }
tokio = { workspace = true }
```

**`crates/voxput-cli/Cargo.toml`:**
```toml
[package]
name = "voxput"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "voxput"
path = "src/main.rs"

[dependencies]
voxput-core = { path = "../voxput-core" }
clap = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
miette = { workspace = true }
serde_json = { workspace = true }

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }
```

**Skeleton `src/lib.rs` for voxput-core:**
```rust
pub mod audio;
pub mod config;
pub mod errors;
pub mod output;
pub mod provider;
pub mod state;
```

**`crates/voxput-cli/src/main.rs`:**
```rust
mod cli;

use clap::Parser;

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = cli::Cli::parse();
    cli::dispatch(cli).await.map_err(|e| miette::miette!("{e}"))
}
```

**Verification:** `cargo build` succeeds, `cargo test` compiles.

---

### Phase 2 — Audio Capture

**Goal:** Record from mic with cpal, encode to WAV bytes with hound.

**`audio/mod.rs`** — trait and shared types:
```rust
pub mod cpal_backend;
pub mod wav;

use crate::errors::Result;

#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,   // raw PCM, mono
    pub sample_rate: u32,
    pub channels: u16,       // always 1
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub is_default: bool,
}

pub trait AudioBackend: Send + Sync {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>>;
    /// `duration_secs` of 0.0 = no time limit; stop flag drives recording end.
    fn record(&self, duration_secs: f32, stop: Arc<AtomicBool>, device_name: Option<&str>) -> Result<AudioData>;
}
```

**`audio/cpal_backend.rs`** — key implementation:
- `CpalBackend` struct implementing `AudioBackend`
- `list_devices()`: `cpal::default_host().input_devices()`, mark default
- `record()`:
  1. Get default (or named) input device
  2. Build `StreamConfig` targeting 16kHz mono f32
  3. Share `Arc<Mutex<Vec<f32>>>` with the data callback
  4. `device.build_input_stream(config, data_callback, err_callback, None)`
  5. `stream.play()` → `std::thread::sleep(duration)` → drop stream
  6. Return `AudioData`
- Capture at the device's native rate if 16kHz is unavailable; resample linearly or use nearest supported rate

**`audio/wav.rs`** — encoding:
```rust
pub fn encode_wav(audio: &AudioData) -> Result<Vec<u8>> {
    // hound::WavWriter writing into a Cursor<Vec<u8>>
    // spec: 16kHz, 16-bit PCM, mono
    // convert f32 samples to i16: (sample * i16::MAX as f32) as i16
}
```

**Verification:**
- Unit test: `encode_wav` with synthetic sine produces valid WAV header bytes
- Manual: write to `/tmp/test.wav`, play with `aplay /tmp/test.wav`

---

### Phase 3 — Groq Whisper API

**Goal:** Send WAV to Groq, receive transcript.

**`provider/mod.rs`:**
```rust
pub mod groq;

use crate::errors::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transcript {
    pub text: String,
    pub language: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct TranscribeOptions {
    pub language: Option<String>,
    pub prompt: Option<String>,
    pub temperature: Option<f32>,
}

#[async_trait::async_trait]
pub trait TranscriptionProvider: Send + Sync {
    async fn transcribe(&self, audio_wav: &[u8], opts: &TranscribeOptions) -> Result<Transcript>;
    fn name(&self) -> &str;
}
```

**`provider/groq.rs`:**
```rust
pub struct GroqProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GroqProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "whisper-large-v3-turbo".into()),
            client: reqwest::Client::new(),
        }
    }
}
```

`transcribe()` implementation:
1. Build `reqwest::multipart::Form` with `file` part (WAV bytes, filename "audio.wav", mime "audio/wav") and `model` part
2. Add optional `language`, `prompt`, `temperature` parts
3. POST to `https://api.groq.com/openai/v1/audio/transcriptions`
4. Header: `Authorization: Bearer {api_key}`
5. On non-2xx: extract error body, return `VoxputError::Provider` with HTTP status context (401 → invalid key, 413 → file too large, 429 → rate limited)
6. Parse JSON `{ "text": "..." }` into `Transcript`

**Verification:**
- Unit test with `mockito`: mock endpoint returns `{"text":"hello"}`, assert `transcript.text == "hello"`
- Integration test with real key: `GROQ_API_KEY=... cargo test --test integration -- groq`

---

### Phase 4 — CLI Wiring

**Goal:** `voxput record` and `voxput devices` work end-to-end.

**`cli/mod.rs`:**
```rust
pub mod devices;
pub mod record;

use clap::{Parser, Subcommand};
use voxput_core::errors::Result;

#[derive(Debug, Parser)]
#[command(name = "voxput", version, about = "Voice-to-text dictation tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Record audio and transcribe to text
    Record(record::RecordArgs),
    /// List available audio input devices
    Devices(devices::DevicesArgs),
}

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Record(args) => record::run(&args).await,
        Commands::Devices(args) => devices::run(&args),
    }
}
```

**`cli/record.rs`:**
```rust
#[derive(Debug, clap::Args)]
pub struct RecordArgs {
    #[arg(long, short, default_value = "5")]
    pub duration: f32,
    #[arg(long, short, default_value = "stdout")]
    pub output: voxput_core::output::OutputTarget,
    #[arg(long)]
    pub device: Option<String>,
    #[arg(long)]
    pub language: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: &RecordArgs) -> Result<()> {
    // 1. load config, get api_key
    // 2. CpalBackend::record(args.duration, args.device)
    // 3. encode_wav(&audio)
    // 4. GroqProvider::transcribe(&wav_bytes, opts)
    // 5. create_sink(&args.output).write(text)
}
```

**`cli/devices.rs`:**
```rust
#[derive(Debug, clap::Args)]
pub struct DevicesArgs {
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: &DevicesArgs) -> Result<()> {
    let devices = CpalBackend.list_devices()?;
    // print as json or plain text with "(default)" marker
}
```

**Progress output convention:** `eprintln!` for "Recording...", "Transcribing..." status; `println!` for transcript (allows piping: `voxput record | xclip`).

**Verification:**
- `cargo run -- --help`
- `cargo run -- devices`
- `GROQ_API_KEY=... cargo run -- record --duration 3`

---

### Phase 5 — Output Sinks

**Goal:** `OutputSink` trait with stdout and clipboard implementations.

**`output/mod.rs`:**
```rust
pub mod clipboard;
pub mod stdout;

use crate::errors::Result;

#[derive(Debug, Clone, Copy, clap::ValueEnum, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputTarget {
    #[default]
    Stdout,
    Clipboard,
    Both,
}

pub trait OutputSink: Send + Sync {
    fn write(&self, text: &str) -> Result<()>;
}

pub fn create_sink(target: &OutputTarget) -> Result<Box<dyn OutputSink>> {
    match target {
        OutputTarget::Stdout => Ok(Box::new(stdout::StdoutSink)),
        OutputTarget::Clipboard => Ok(Box::new(clipboard::ClipboardSink)),
        OutputTarget::Both => Ok(Box::new(BothSink)),
    }
}

struct BothSink;
impl OutputSink for BothSink {
    fn write(&self, text: &str) -> Result<()> {
        stdout::StdoutSink.write(text)?;
        clipboard::ClipboardSink.write(text)
    }
}
```

**`output/clipboard.rs`:** use `arboard::Clipboard`, map errors to `VoxputError::Output`. Print `"Copied to clipboard."` to stderr.

**Verification:**
- `cargo run -- record --output clipboard --duration 3` → paste result
- `cargo run -- record --output both --duration 3`
- Unit tests; clipboard test marked `#[ignore]` for headless CI

---

### Phase 6 — State Machine & Error Handling

**Goal:** `DictationStateMachine` governing lifecycle; rich error messages.

**`state.rs`:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationState { Idle, Recording, Transcribing, Error }

pub enum DictationEvent {
    StartRecording,
    StopRecording,
    TranscriptionComplete(String),
    TranscriptionFailed(String),
    Reset,
}

pub struct DictationStateMachine {
    state: DictationState,
    last_error: Option<String>,
    last_transcript: Option<String>,
}

impl DictationStateMachine {
    pub fn new() -> Self { ... }
    pub fn state(&self) -> DictationState { ... }
    pub fn handle(&mut self, event: DictationEvent) -> DictationState { ... }
    // Invalid transitions: tracing::warn!, do not panic
}
```

The CLI record command transitions through: `StartRecording` → `StopRecording` → `TranscriptionComplete`/`TranscriptionFailed`. This is lightweight in the MVP but establishes the pattern the daemon will use with async channels.

**`errors.rs`** (using thiserror + miette, same pattern as untangle's `src/errors.rs`):
```rust
#[derive(Error, Diagnostic, Debug)]
pub enum VoxputError {
    #[error("Audio error: {0}")]
    #[diagnostic(code(voxput::audio))]
    Audio(String),

    #[error("No audio input device available")]
    #[diagnostic(code(voxput::no_device), help("Check that a microphone is connected and accessible"))]
    NoDevice,

    #[error("Transcription provider error: {0}")]
    #[diagnostic(code(voxput::provider))]
    Provider(String),

    #[error("API key not found: set {env_var} or add to ~/.config/voxput/config.toml")]
    #[diagnostic(code(voxput::missing_api_key))]
    MissingApiKey { env_var: String },

    #[error("Configuration error: {0}")]
    #[diagnostic(code(voxput::config))]
    Config(String),

    #[error("Output error: {0}")]
    #[diagnostic(code(voxput::output))]
    Output(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VoxputError>;
```

**Verification:**
- Unit tests: all valid state transitions
- Unit tests: invalid transitions do not panic, emit tracing warn
- proptest: random event sequences never panic, always yield a valid state

---

### Phase 7 — Configuration

**Goal:** Layered config: defaults → `~/.config/voxput/config.toml` → env vars → CLI flags.

**`config/schema.rs`** — TOML file format:
```toml
provider = "groq"

[providers.groq]
api_key_env = "GROQ_API_KEY"   # env var name (default)
api_key = ""                    # direct key (not recommended)
model = "whisper-large-v3-turbo"

[audio]
device = "default"
sample_rate = 16000

[output]
target = "stdout"
```

**`config/mod.rs`:**
```rust
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub api_key_env: String,
    pub api_key: Option<String>,    // resolved from env or file
    pub model: Option<String>,
    pub provider: String,
    pub device: Option<String>,
    pub sample_rate: u32,
    pub output_target: String,
}

impl ResolvedConfig {
    /// Resolve API key: file field first, then env var.
    pub fn api_key(&self) -> Result<String> { ... }
}

/// Config file: ~/.config/voxput/config.toml (via dirs::config_dir())
pub fn load_config() -> Result<ResolvedConfig> {
    // 1. built-in defaults
    // 2. read TOML file if present
    // 3. override from GROQ_API_KEY, VOXPUT_MODEL env vars
}
```

**Verification:**
- Unit test: `load_config()` with no file returns defaults
- Unit test: env var `GROQ_API_KEY` is resolved by `api_key()`
- Unit test: TOML file fields override defaults

---

## 4. Dependency Summary

| Phase | New workspace dependencies |
|-------|---------------------------|
| 1. Scaffolding | tokio, reqwest, serde, serde_json, toml, cpal, hound, arboard, thiserror, miette, async-trait, tracing, tracing-subscriber, clap, dirs |
| 2. Audio | (none — cpal, hound already declared) |
| 3. Provider | (none — reqwest, async-trait already declared) |
| 4–7 | (none new) |
| Dev | insta, assert_cmd, predicates, tempfile, mockito, proptest |

---

## 5. Testing Strategy

Matching patterns from the Untangle project:

**Unit tests** (`#[cfg(test)] mod tests` in each module):
- `audio/wav.rs`: encode synthetic samples, verify WAV header bytes
- `provider/groq.rs`: use `mockito` to mock Groq endpoint
- `output/stdout.rs`: writes correctly
- `config/schema.rs`: TOML round-trip for full/partial/empty configs
- `config/mod.rs`: defaults, file override, env override
- `state.rs`: all transitions, invalid transition handling

**Integration tests** (`tests/` directory, matching Untangle's `tests/integration.rs` + subdirectory pattern):
```
tests/
  integration.rs       # mod integration { mod record_test; mod devices_test; }
  integration/
    record_test.rs     # assert_cmd tests: --help, --version, bad API key error
    devices_test.rs    # assert_cmd: devices lists ≥1 device or fails gracefully
```

**Property-based tests** (proptest in `state.rs`):
- Random event sequences never panic, always yield a valid `DictationState`

**Snapshot tests** (insta):
- JSON output of `voxput record --json` with mocked provider
- JSON output of `voxput devices --json`

---

## 6. Key Design Decisions

**Two-crate workspace from day one:** Even though the MVP is a CLI, `voxput-core` has no clap dependency. The future daemon (`voxputd`) will depend on `voxput-core` without any CLI baggage.

**Traits from day one** (`AudioBackend`, `TranscriptionProvider`, `OutputSink`): Matches Untangle's `ParseFrontend` trait pattern. Makes unit testing easy (mock implementations) and future backends drop in cleanly.

**State machine as plain sync struct:** `handle(event) -> state` is synchronous. The MVP drives it synchronously in the CLI. The daemon will drive it from a tokio task via a `mpsc` channel. No async machinery required in the state logic itself.

**WAV encoding in memory:** Avoids filesystem I/O, temp file cleanup, and permission issues. Groq's 25MB limit ≈ 13 minutes of 16kHz mono 16-bit audio — far beyond typical dictation.

**Capture at 16kHz mono:** Groq downsamples to 16kHz anyway. Capturing at this rate minimizes file size.

**`eprintln!` for status, `println!` for data:** Follows Unix convention; allows piping `voxput record | xclip` or `voxput record > file.txt` without status noise.

**Config resolution order** (matching Untangle's `config/resolve.rs` pattern): built-in defaults → `~/.config/voxput/config.toml` → env vars → CLI flags.

---

## 7. Post-MVP Roadmap

### Phase 8 — Daemon (`voxputd`) with IPC

> **What actually shipped (v0.2):** D-Bus session service using `zbus`
> instead of gRPC. The D-Bus interface (`com.github.jonochang.Voxput1`)
> exposes `StartRecording`, `StopRecording`, `Toggle`, `GetStatus` methods
> and a `StateChanged` signal. gRPC was not needed — D-Bus integrates
> natively with GNOME Shell and systemd.

New crate `crates/voxputd/` with `tonic` + `prost` gRPC server over a Unix domain socket (`$XDG_RUNTIME_DIR/voxput.sock`).

**Proto definition (`proto/voxput.proto`):**
```protobuf
service VoxputService {
  rpc StartRecording(StartRequest) returns (StartResponse);
  rpc StopRecording(StopRequest) returns (StopResponse);
  rpc GetStatus(StatusRequest) returns (StatusResponse);
  rpc SubscribeState(Empty) returns (stream StateUpdate);
}
```

The CLI gains `voxput start`, `voxput stop`, `voxput toggle`, `voxput status` subcommands that connect to the daemon via gRPC.

Daemon ships as a systemd user service: `contrib/voxputd.service`.

### Phase 9 — Push-to-Talk Hotkey

> **What actually shipped (v0.3):** The GNOME extension communicates with
> `voxputd` over D-Bus directly (no bridge needed). Key-release detection
> uses `Main.pushModal()` to grab keyboard input — `global.stage`
> key-release events are not delivered on Wayland when an app has focus.

GNOME Shell extension (`extensions/gnome/`) registers a keyboard shortcut via GNOME's Keybindings API. On key-down → call daemon `StartRecording`; on key-up → `StopRecording`. The extension communicates with the daemon via a small D-Bus bridge or helper binary that forwards to gRPC.

### Phase 10 — Full GNOME Shell Extension

> **What actually shipped (v0.3):** All items below are implemented. The
> extension also supports toggle mode (press-to-start, press-to-stop) as an
> alternative to push-to-talk mode, switchable via GSettings. A Nix flake
> with Home Manager module manages the full installation.

- Top-bar indicator with state icons (idle / recording / transcribing / error)
- Popup menu: status, last transcript preview, settings shortcut, toggle
- GNOME Settings panel for hotkey configuration
- D-Bus adapter bridging GNOME Shell ↔ daemon gRPC

### Phase 11 — Type-at-Cursor Output

> **What actually shipped (v0.3):** Auto-paste via `ydotool`, which injects
> keystrokes at the kernel uinput level. `wtype` does not work on GNOME
> (requires wlroots). Clutter's virtual keyboard API crashes gnome-shell.
> Clipboard is always set as fallback regardless of ydotool availability.

New `TypeAtCursorSink` implementing `OutputSink`:
- Linux Wayland: `wtype` subprocess, or `xdg-desktop-portal` `RemoteDesktop` portal
- Linux X11 fallback: `xdotool type`
- Always fall back to clipboard if type-at-cursor fails
- Config: `output.target = "type"` or `--output type`

### Phase 12 — Additional Providers

- `provider/openai.rs`: OpenAI-compatible endpoint (same multipart API)
- `provider/local_whisper.rs`: shell out to `whisper.cpp` CLI or use `whisper-rs` bindings
- Provider factory following Untangle's `parse/factory.rs` pattern
- Config: `provider = "openai"` or `provider = "local_whisper"`
- Hybrid flow: local STT + optional LLM cleanup via Ollama

### Phase 13 — Cross-Platform

- `cpal` and `arboard` already abstract audio and clipboard
- New platform output sinks: macOS CGEvent typing, Windows SendInput
- macOS menu bar app (Swift or Tauri)
- Windows tray app (Tauri or native)
- `launchd` agent (macOS), startup task (Windows)
- Platform-specific packaging (`.dmg`, `.msi`)
