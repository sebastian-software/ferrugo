# 0041: Renderer Gap Triage And Support Matrix

Status: done
Phase: 5
Size: small
Depends on: 0040

## Goal

Turn the typical-document coverage gate into a prioritized native renderer
support matrix.

## Scope

- Classify failures from the typical corpus by missing PDF feature.
- Separate product-visible rendering gaps from parser, IO, and harness gaps.
- Define support levels: rendered, degraded, unsupported, malformed, and
  encrypted.
- Rank gaps by document frequency, thumbnail impact, implementation risk, and
  memory risk.

## Non-Goals

- Implement new renderer features.
- Claim full PDF compatibility.
- Remove PDFium fallback paths.

## Deliverables

- Native renderer support matrix.
- Ranked implementation backlog for the next renderer gaps.
- Updated unsupported-feature taxonomy if new classes are needed.

## Acceptance Criteria

- Each corpus failure has a stable category and owner milestone.
- The next rendering work is ordered by measured product value.
- The matrix explicitly states where PDFium remains required.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Re-run the typical-document corpus comparison command.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `docs/reports/native-renderer-support-matrix-2026-06-24.md` with
  support levels, seed fixture status, local corpus categories, PDFium fallback
  requirements, and ranked renderer gaps.
- Updated `docs/errors.md` with internal native unsupported-feature buckets
  while keeping the public thumbnail error taxonomy unchanged.
- Classified 0040 failures:
  - `form-xobject.pdf` is `unsupported` under
    `renderer.form-xobject-composition`; it should be pulled forward before
    the font pipeline, with final facade parity still covered by 0059.
  - `inline-image.pdf` is `unsupported` under
    `renderer.inline-image-stream`; it should be pulled forward before 0047
    image filter coverage.
- Confirmed PDFium remains required for faithful text, Form XObject
  composition, inline images, real-world image filters, transparency, encrypted
  files, and malformed recovery until the later milestones land.
- Validation:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/page-size-letter.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-page-size-letter-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/vector-paths.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-vector-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/text-page.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-text-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/image-xobject.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-image-xobject-native.png`
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/form-xobject.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-form-native.png` returned `render error [unsupported]`.
  - `cargo run -p ferrugo-cli -- render-native fixtures/generated/inline-image.pdf --max-edge 256 --output target/ferrugo-thumbnails/coverage-inline-image-native.png` produced a blank native image with `nonwhite=0`.
  - `cargo clippy --all-targets --all-features -- -D warnings`
