# 0135: Renderer Diagnostics And Debug Artifact Bundle

Status: done
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

- Added opt-in `summarize-fallbacks --diagnostics-dir <path>` integration that
  writes one JSON diagnostic bundle per fallback-required/error fixture.
- Bundle format records render options, manifest metadata, safe page
  count/page sizes, metadata/render timings, typed error class/category, coarse
  stage hints, and native memory diagnostics.
- Default bundles exclude PDF bytes, rendered pixels, and document-info fields.
- Smoke artifact:
  `target/diagnostics-0135/0004-fixtures-generated-optional-content-ocmd-pdf.diagnostics.json`.
- Report: `docs/reports/renderer-diagnostics-bundle-2026-06-25.md`.
- Validation:
  - `cargo fmt --check`
  - `cargo test -p ferrugo-cli diagnostic_bundles -- --nocapture`
  - `cargo test -p ferrugo-cli fallback_summary_config_should_accept_family_filters -- --nocapture`
  - `cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family presentation --max-edge 160 --diagnostics-dir target/diagnostics-0135 --output target/diagnostics-0135-summary.json`
  - `cargo check --workspace`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `cargo test --workspace --no-default-features`
