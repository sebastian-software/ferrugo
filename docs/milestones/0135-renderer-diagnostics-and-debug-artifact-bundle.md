# 0135: Renderer Diagnostics And Debug Artifact Bundle

Status: todo
Phase: 24
Size: medium
Depends on: 0134

## Goal

Make native rendering failures easier to diagnose by emitting compact, safe
debug artifacts for parser, display-list, raster, and comparison stages.

## Scope

- Add opt-in diagnostic bundles for failing fixtures and corpus runs.
- Include renderer options, page metadata, typed errors, and stage timings.
- Keep private PDF bytes and rendered pages out of artifacts unless explicitly
  requested.
- Add redaction guidance for sharing diagnostics.

## Non-Goals

- Log private document contents by default.
- Build a graphical debugger.
- Replace targeted unit tests with artifact inspection.

## Deliverables

- Diagnostic bundle format.
- CLI/report integration for failed corpus entries.
- Privacy notes for artifact sharing.

## Acceptance Criteria

- A failing corpus entry can produce a useful diagnostic bundle.
- Artifacts are deterministic enough for regression tracking.
- Sensitive data is excluded by default and documented when optional.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run diagnostic bundle smoke tests.
- Run corpus comparison with diagnostics enabled.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
