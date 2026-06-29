# 0198: Native Renderer Telemetry Privacy And Diagnostics Policy

Status: done
Phase: 37
Size: small
Depends on: 0197

## Goal

Define privacy-safe renderer diagnostics so consumers can report failures,
unsupported features, and performance issues without leaking document content.

## Scope

- Classify diagnostic fields as safe, sensitive, local-only, or experimental.
- Add redaction rules for fixture paths, metadata, text snippets, and object
  identifiers.
- Document how consumers should attach debug bundles to issue reports.
- Ensure telemetry remains optional and application-controlled.

## Non-Goals

- Add hosted telemetry collection.
- Log document text or image content by default.
- Make diagnostics required for rendering.

## Deliverables

- Telemetry and diagnostics privacy policy.
- Redaction checklist for debug artifacts.
- Tests or review checks for generated diagnostic bundles.

## Acceptance Criteria

- Diagnostics avoid document content unless explicitly local-only.
- Unsupported and budget outcomes remain reportable.
- Public docs describe safe issue-reporting data.

## Validation

- Review generated debug artifacts.
- Run diagnostic bundle tests if available.
- Run native-only `cargo test`.
- Audit docs for privacy-sensitive examples.

## Completion Notes

- Added `docs/policies/telemetry-diagnostics-privacy.md` with safe,
  sensitive, local-only, and experimental field classes.
- Diagnostic bundles now declare telemetry as application-controlled and
  disabled by default.
- Diagnostic bundles now declare privacy field classes and redact
  private/local-only paths and manifest details.
- Updated native backend and trace docs to route privacy guidance through the
  central policy.
- Report:
  `docs/reports/native-renderer-telemetry-privacy-2026-06-29.md`.
- Validation:
  - `cargo test -p pdfrust-cli diagnostic_bundles -- --nocapture`
  - `cargo fmt --check`
  - `git diff --check -- crates/pdfrust-cli/src/main.rs docs/backend/native.md docs/policies/native-render-trace.md docs/policies/telemetry-diagnostics-privacy.md docs/milestones/0198-native-renderer-telemetry-privacy-and-diagnostics-policy.md docs/milestones/README.md docs/reports/native-renderer-telemetry-privacy-2026-06-29.md`
  - `cargo test --workspace --no-default-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
