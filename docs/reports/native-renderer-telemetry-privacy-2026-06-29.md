# Native Renderer Telemetry Privacy 2026-06-29

Milestone 0198 centralizes privacy rules for renderer diagnostics and makes the
diagnostic bundle schema explicit about telemetry control and field classes.

## Implementation

- Added `docs/policies/telemetry-diagnostics-privacy.md`.
- Diagnostic bundles now include:
  - `telemetry.collection = "none"`;
  - `telemetry.controlled_by = "application"`;
  - `telemetry.default_enabled = false`;
  - boolean privacy guarantees for PDF bytes, rendered pixels, document-info
    fields, text samples, and private paths;
  - field classes for path, manifest, options, metadata, stages, and memory
    diagnostics.
- Private/local-only manifest entries now redact diagnostic bundle path IDs to
  `local-only-NNNN`.
- Private/local-only manifest details are emitted as
  `{"status":"redacted","reason":"privacy-sensitive-fixture"}` instead of
  source, license, features, or notes.
- Native backend and native trace docs now link to the central privacy policy.

## Bundle Review

The diagnostic bundle unit test covers both a committed fixture and a synthetic
private/local-only manifest entry. It verifies that bundles:

- declare telemetry as disabled by default;
- declare no PDF bytes, rendered pixels, document-info fields, text samples, or
  private paths;
- preserve typed unsupported category data;
- redact private paths and manifest notes;
- avoid raw `%PDF` bytes.

## Validation Commands

```text
cargo test -p pdfrust-cli diagnostic_bundles -- --nocapture
cargo fmt --check
git diff --check -- crates/pdfrust-cli/src/main.rs docs/backend/native.md docs/policies/native-render-trace.md docs/policies/telemetry-diagnostics-privacy.md docs/milestones/0198-native-renderer-telemetry-privacy-and-diagnostics-policy.md docs/milestones/README.md docs/reports/native-renderer-telemetry-privacy-2026-06-29.md
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
