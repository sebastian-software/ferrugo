# 0062: PDFium Fallback Telemetry And Kill Switch

Status: done
Phase: 9
Size: small
Depends on: 0061

## Goal

Measure and control every remaining PDFium fallback so replacement progress is
visible and reversible.

## Scope

- Emit structured fallback reasons from automatic backend selection.
- Add a strict native-only mode for CI and release validation.
- Add a fallback deny-list or kill switch for targeted experiments.
- Track fallback counts by unsupported feature category.

## Non-Goals

- Upload telemetry to a remote service.
- Change renderer output quality.
- Remove fallback paths before evidence supports it.

## Deliverables

- Fallback reason taxonomy.
- Native-only validation mode.
- Documentation for interpreting fallback diagnostics.

## Acceptance Criteria

- Every automatic PDFium fallback has a typed reason.
- CI can fail when native-only rendering regresses on supported categories.
- Local corpus runs summarize fallback volume by category.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run native-only fixture validation.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added optional native unsupported-feature buckets while preserving the public
  `unsupported` class.
- Automatic native-first rendering now reports structured fallback diagnostics
  (`fallback_reason` and `fallback_category`).
- Added `--native-only` / `--no-pdfium-fallback`,
  `--deny-fallback-reason <bucket>`, `FERRUGO_NATIVE_ONLY=1`, and
  `FERRUGO_DENY_FALLBACK_REASONS=...` to block all or targeted PDFium
  fallbacks.
- Added `summarize-fallbacks` for local corpus runs. On 2026-06-24,
  `fixtures/generated` at `--max-edge 120` reported 39 total PDFs, 37 native
  renders, 1 fallback required in `graphics.optional-content`, and 1 encrypted
  input.
- Validation: `cargo fmt --check`, `cargo check`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test --quiet`,
  `cargo run -p ferrugo-cli -- render fixtures/generated/vector-paths.pdf --native-only --max-edge 120 --output target/native-only-vector.png`,
  and
  `cargo run -p ferrugo-cli -- summarize-fallbacks fixtures/generated --max-edge 120 --output target/fallback-summary.json`.
