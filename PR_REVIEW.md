# Code Review (Historical)

> **Note:** This review was written during the v0.1 MVP phase. Some findings
> have been addressed in subsequent releases; others may still be relevant.
> Preserved as a historical reference.

Review scope: `docs/specs/*`, `implementation.md`, `README.md`, and current `crates/` code.

**Summary**
The codebase is clean and small, with a clear MVP focus (CLI-only). The biggest issues are doc/code drift, unused config pathways, and a few design mismatches that will create confusion as features expand. The core architecture is reasonable, but some defaults are duplicated and some features are implied but not wired.

**Strengths**
- Clear separation between `voxput-core` and `voxput-cli` matches the intended layering and keeps the CLI thin.
- Core components (audio, provider, output, config, state) are minimal and testable.
- Errors are unified and user-facing messages are mostly actionable.

**Findings: Simplicity & Redundancy**
- Config values are partially unused. `ResolvedConfig.output_target` and `ResolvedConfig.sample_rate` are loaded but never applied. `ResolvedConfig.provider` is unused and only Groq is supported. This adds conceptual surface area without runtime effect. (`crates/voxput-core/src/config/mod.rs`, `crates/voxput-cli/src/cli/record.rs`)
- `DictationStateMachine` is instantiated in `record::run` but its state is not used to drive behavior or output, making it dead weight for the CLI path. Either integrate it with logging/status output or move it behind the future daemon. (`crates/voxput-cli/src/cli/record.rs`, `crates/voxput-core/src/state.rs`)
- Defaults are duplicated across docs and code. `README.md` and `implementation.md` each describe defaults (model, sample rate, output target), but the authoritative defaults live in code. This increases drift risk.

**Findings: Maintainability**
- Doc drift in `implementation.md`:
  - It says WAV encoding uses `hound`, but the code uses a manual encoder. (`crates/voxput-core/src/audio/wav.rs`)
  - It describes resampling or “nearest supported rate,” but `CpalBackend` only prefers 16 kHz and otherwise uses the device default with no resampling. (`crates/voxput-core/src/audio/cpal_backend.rs`)
  - It describes `tests/integration/*` that do not exist, while actual integration tests live in `crates/voxput-core/tests/`. (`implementation.md` vs repo layout)
  - It shows `create_sink(&OutputTarget) -> Result<...>` but current API is `create_sink(OutputTarget) -> Box<dyn OutputSink>`.
- `VOXPUT_MODEL` is supported as an env override but is not documented in `README.md`, and config output target is documented but unused by CLI. (`crates/voxput-core/src/config/mod.rs`, `README.md`)
- `crates/voxput-core/tests/groq_integration.rs` always runs a live network call for the “invalid key” test, even if `GROQ_API_KEY` is unset. This makes CI network-dependent and flaky. Consider gating all live calls behind a `GROQ_API_KEY` check or a feature flag.

**Findings: Design Alignment**
- `docs/specs/design.md` describes a multi-layer daemon + IPC design, while the current implementation and `implementation.md` are CLI-only. That’s fine for MVP, but the docs read like current architecture. A clear “current vs future” header would reduce confusion.
- `docs/specs/background.md` and `docs/specs/design.md` use nonstandard formatting (e.g., `⸻`, inline bullets, prefixed `<Right>`). This hurts Markdown rendering and maintainability, and will make future diffs noisy.

**Recommended Changes (Highest Impact First)**
1. Decide what config is actually supported in MVP and either wire it in or remove it.
   - Wire `output_target` and `sample_rate` into `voxput record`, or drop them from config/schema and README for now.
2. Fix doc drift in `implementation.md` to match the current code (WAV encoder, test locations, sink API, resampling behavior).
3. Make integration tests deterministic in CI:
   - Gate all live Groq calls behind `GROQ_API_KEY`, or move them to an ignored test group.
4. Clarify in `docs/specs/design.md` which sections are “future vision” vs “current architecture.”
5. If keeping `DictationStateMachine` in MVP, integrate it with real behavior (e.g., status events or logs). Otherwise move it behind a `voxputd` feature or remove from CLI path.

**Optional Improvements**
- Add request timeouts to the Groq `reqwest::Client` to avoid hanging calls.
- Parse `output_target` from config into `OutputTarget` enum to centralize validation.
- Expose a single source of defaults (e.g., `ResolvedConfig::default()` used by CLI, and docs derived from it).
