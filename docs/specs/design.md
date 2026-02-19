Architecture

Core principle: split Voxput into a portable Rust core and thin platform front-ends. GNOME is just one front-end; later you can add macOS, Windows, KDE, etc. without rewriting providers/audio/transcription logic.

Layers:
	1.	voxput-core (portable Rust library)

	•	Audio capture abstraction (platform backends)
	•	Provider abstraction (Groq now; local Whisper/Ollama later)
	•	Post-processing pipeline (optional)
	•	Output abstraction (stdout/clipboard/type-at-cursor)
	•	State machine (Idle/Recording/Transcribing/Error)

	2.	voxputd (local service/daemon, cross-platform)

	•	Runs continuously
	•	Hosts the state machine + orchestrates audio → provider → output
	•	Exposes an IPC API to front-ends
	•	Emits state updates for indicators/UX

	3.	Front-ends (platform-specific UX + shortcut registration + indicator)

	•	GNOME Shell extension (Wayland-safe hotkeys + indicator)
	•	KDE / Plasma widget or KGlobalAccel integration
	•	macOS menu bar app
	•	Windows tray app
	•	Optional: cross-platform tray (Tauri) for non-DE-specific environments

	4.	voxput CLI

	•	Scriptable entrypoint, prints to stdout
	•	Can control voxputd via IPC or run one-shot mode directly

This gives you:
	•	One core implementation (providers/audio/pipeline/output)
	•	Many UI shells (indicator + hotkey + settings)

⸻

Technologies

Shared (all platforms)
	•	Language: Rust
	•	Async: tokio
	•	HTTP: reqwest
	•	Serialization: serde, serde_json
	•	CLI: clap
	•	Logging: tracing
	•	Config: figment or config, plus dirs

IPC (cross-platform approach)
Use two IPC options so you can support everything cleanly:

A) Local gRPC over Unix domain socket / named pipe (recommended cross-platform baseline)
	•	Linux/macOS: Unix domain socket
	•	Windows: named pipe
	•	Benefits: consistent API across all OSes, great for CLI + tray apps + editor plugins
	•	Rust options: tonic (gRPC) or tarpc / capnp / ipc-channel

B) Native IPC bridge (optional, per platform)
	•	Linux desktops: D-Bus (best integration for GNOME/KDE)
	•	macOS: XPC (nice but optional)
	•	Windows: COM (not necessary if you have named pipes)

Pragmatic recommendation: implement gRPC as the canonical control API for voxputd, and provide D-Bus adapter on Linux later if you want “native” integration. GNOME extension can talk to a small helper that bridges D-Bus ↔ gRPC.

Audio capture backends
Abstract audio capture behind AudioBackend:
	•	Linux: PipeWire (Wayland-first)
	•	macOS: CoreAudio (via cpal or native bindings)
	•	Windows: WASAPI (via cpal or native bindings)

MVP approach: use cpal for cross-platform capture, and add PipeWire-specific tuning later on Linux if needed.

Output backends (text insertion)
Abstract behind OutputSink:
	•	StdoutSink (always)
	•	ClipboardSink (cross-platform via arboard)
	•	TypeAtCursorSink (platform-specific)
	•	Linux Wayland: wtype (fallback), portals when possible
	•	Linux X11: xdotool
	•	macOS: Accessibility APIs / CGEvent (or AppleScript fallback)
	•	Windows: SendInput / UI Automation

Important: “type at cursor” is the hardest to do perfectly across OSes; always have clipboard fallback.

Indicators / tray / menu bar
	•	GNOME: Shell extension indicator
	•	KDE: Plasma widget / KStatusNotifierItem
	•	macOS: menu bar app (Swift or Tauri)
	•	Windows: tray icon app (WinUI, Tauri, or Rust + native)

To avoid rewriting UI multiple times, consider a cross-platform tray app later:
	•	Tauri (Rust core + JS UI) is a strong fit for tray + global shortcuts
	•	But GNOME hotkeys on Wayland are still better via DE integration

⸻

Language & Dependencies (Rust) — revised

Core crates
	•	tokio, reqwest, serde, serde_json, tracing
	•	thiserror, anyhow (error handling)
	•	cpal (cross-platform audio capture)
	•	hound or wav (encode audio for Whisper endpoints)
	•	arboard (clipboard)
	•	tonic + prost (gRPC API)
	•	Optional: notify-rust (Linux notifications), platform-specific later

Platform modules
	•	voxput-platform-linux (PipeWire tuning, Wayland typing)
	•	voxput-platform-macos (menu bar helper, input injection)
	•	voxput-platform-windows (tray helper, input injection)

⸻

Subcomponents (updated)

Providers (extensible)
Keep the same provider trait, but explicitly support:
	•	Cloud STT providers (Groq, OpenAI, Deepgram, etc.)
	•	Local STT (whisper.cpp, faster-whisper)
	•	Hybrid flows (local STT + optional LLM clean-up via Ollama)

Provider interface:
	•	transcribe(audio, opts) -> Transcript
	•	Optional streaming later:
	•	transcribe_stream(chunks) -> partial results

Config example

provider = "groq"

[providers.groq]
api_key_env = "GROQ_API_KEY"
model = "whisper-large-v3-turbo"

[providers.local_whisper]
backend = "whisper.cpp"
model_path = "/…/ggml-large.bin"

Keyboard shortcuts (customisable, cross-platform)
Define a cross-platform “control intent”:
	•	PushToTalkDown
	•	PushToTalkUp
	•	ToggleRecording

Where shortcuts live
	•	On DEs that support it (GNOME/KDE): register via native mechanisms
	•	On macOS/Windows: register via tray app or native global hotkey APIs
	•	For headless: let user bind via their window manager, calling voxput toggle

Implementation options
	•	GNOME: Shell extension handles it
	•	KDE: KGlobalAccel integration (or Plasma widget)
	•	Cross-platform tray: Tauri global shortcut APIs (later)
	•	CLI: no global key handling, just commands

CLI (stdout-first)
Unchanged, but now it’s explicitly portable.
	•	voxput record prints to stdout
	•	voxput record --json
	•	voxput start/stop/toggle/status via gRPC
	•	voxputd can run on macOS/Windows too

⸻

Packaging / lifecycle (future-proof)
	•	Linux: systemd user service + extension/widget
	•	macOS: launchd agent + menu bar app
	•	Windows: startup task/service + tray app

⸻

Summary: what changes vs GNOME-only design
	•	The “daemon + core” becomes OS-agnostic.
	•	GNOME extension becomes one optional front-end, not the foundation.
	•	IPC becomes gRPC/named-pipe/socket as the primary interface.
	•	Audio and typing become backend traits with per-OS implementations.
	•	KDE/macOS/Windows get their own thin UI layers later.

⸻

If you want, I can also propose a v1/v2 roadmap aligned to this:
	•	v1: Linux GNOME + CLI + Groq + clipboard fallback
	•	v1.5: KDE/Plasma + better Wayland typing
	•	v2: macOS menu bar + Windows tray + local whisper.cpp provider
