# 0018: Live Thumbnail Fixture Render

Status: done
Phase: 1
Size: small
Depends on: 0017

## Goal

Render the generated fixture PDFs through the PDFium backend and CLI.

## Scope

- Render at least `fixtures/generated/text-page.pdf`.
- Write one PNG to `target/`.
- Record dimensions, render time, and memory for `max_edge` 256, 512, and 1024.
- Replace placeholder baseline fields with real values.

## Non-Goals

- Build a full visual regression suite.
- Commit large pixel artifacts.
- Tune rendering performance.

## Deliverables

- Updated measurement report.
- Updated baseline example with real dimensions and pixel digest.
- CLI command transcript or summary.

## Acceptance Criteria

- CLI writes a valid PNG for a generated fixture.
- Output dimensions honor `max_edge`.
- Baseline metadata references the generated fixture and PDFium backend.

## Validation

- Inspect PNG dimensions.
- Re-run `cargo test` and `cargo clippy`.

## Completion Notes

Completed on 2026-06-24.

- Rendered `fixtures/generated/text-page.pdf` through the release CLI with the
  live PDFium backend.
- Wrote PNG artifacts under `target/pdfrust-thumbnails/`.
- Recorded dimensions, file hashes, decoded RGBA hashes, wall time, CPU time,
  and max RSS for `max-edge` 256, 512, and 1024 in
  `docs/measurements/pdfium-build-baseline.md`.
- Replaced the placeholder success baseline with the real 256px fixture
  dimensions and digest.
- `max-edge` 512 and 1024 produced identical 300x160 output because the fixture
  is not upscaled.
