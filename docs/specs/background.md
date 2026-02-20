# Voxput — Background Brief

## 1. Background and Problem

Modern developers and knowledge workers increasingly use voice dictation to accelerate writing, coding, and prompting AI systems. Tools like FreeFlow provide a seamless "hold-to-talk, paste at cursor" experience on macOS, but there is no equivalent solution that feels native, reliable, and well-integrated on GNOME/Linux.

Current limitations on Linux include:

- Lack of polished, GNOME-native dictation tools
- Poor integration with Wayland (global hotkeys and input injection are restricted)
- Inconsistent clipboard/paste behavior across environments
- No clean equivalent to macOS-style menu bar status indicators
- Fragmented solutions (scripts, CLI tools, ad hoc hotkey bindings)

Additionally, tools that rely on macOS-specific behaviors (e.g., Cmd+V paste simulation) do not translate well to GNOME, especially under Wayland.

There is an opportunity to build a modern, Rust-based, GNOME-native dictation tool that:

- Uses Groq's Whisper API for fast transcription
- Provides reliable push-to-talk functionality
- Displays a clear recording indicator in the GNOME top bar
- Inserts text into the focused application in a predictable way

Voxput aims to fill this gap.

---

## 2. Goal

The goal of Voxput is to provide a lightweight, GNOME-native background daemon that enables seamless voice-to-text insertion into any focused application.

Specifically, Voxput should:

- Run as a background service (`voxputd`)
- Display a GNOME top-bar indicator showing: Idle, Recording, Transcribing, Error
- Support push-to-talk via a global keybinding
- Capture microphone audio using PipeWire
- Send audio to Groq Whisper API for transcription
- Insert the resulting text at the current cursor location (or copy to clipboard)
- Work reliably on GNOME Wayland
- Be written in Rust for performance, safety, and maintainability

The intended user experience:

1. User holds hotkey.
2. Indicator shows recording state.
3. User releases hotkey.
4. Indicator shows transcribing.
5. Text appears at the cursor.
6. Indicator returns to idle.

The experience should feel immediate, minimal, and unobtrusive — comparable to FreeFlow on macOS.

---

## 3. Constraints

### Platform Constraints

- Must work on GNOME (Wayland-first environment).
- Cannot rely on low-level global key capture in userland (Wayland restrictions).
- Input injection requires kernel-level tools (`ydotool`) or clipboard fallback — compositor-level virtual keyboards (Clutter) and wlroots-only tools (`wtype`) do not work reliably on GNOME Wayland.
- GNOME Shell does not support legacy system trays by default; indicator should be implemented via a Shell extension.

### Architectural Constraints

- Hotkey handling should use GNOME Shell APIs rather than unsafe global hooks.
- Audio capture should use PipeWire (modern GNOME standard).
- Communication between UI and daemon should use D-Bus.
- Daemon should run as a user-level systemd service.

### UX Constraints

- Indicator must clearly communicate state (recording/transcribing).
- Latency must feel minimal (visual feedback should be instant on keypress).
- Failure modes (network error, mic unavailable, API failure) must be visible and understandable.
- Must not disrupt normal keyboard shortcuts.

### API Constraints

- Must securely handle `GROQ_API_KEY`.
- Should support model configurability (e.g., whisper-large-v3, turbo variants).
- Should degrade gracefully if API is unavailable.

### Design Constraints

- Minimal UI surface area.
- No heavy desktop window.
- Background-first, menu-driven interaction.
- Fast startup and low idle resource usage.
