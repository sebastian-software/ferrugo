# 0172: High-DPI Thumbnail And Preview Fidelity

Status: done
Phase: 32
Size: medium
Depends on: 0171

## Goal

Improve thumbnail and preview output at higher device scales while preserving
bounded memory use and deterministic native rendering.

## Scope

- Add high-DPI render fixtures and visual thresholds for text, vector edges,
  image scaling, and transparency.
- Audit scale-dependent cache keys and raster buffer allocation.
- Fix high-DPI artifacts in accepted document families.
- Document supported scale limits and memory behavior.

## Non-Goals

- Render arbitrary poster-sized pages at unlimited scale.
- Add GPU-specific output paths.
- Change low-DPI behavior without regression evidence.

## Deliverables

- High-DPI fidelity report.
- Scale-aware cache and raster fixes.
- Updated preview and thumbnail docs.

## Acceptance Criteria

- High-DPI thumbnails preserve expected text, vector, and image fidelity.
- Scale-dependent caches cannot return stale low-DPI content.
- Render scale limits are explicit and enforced.

## Validation

- Run native-only `cargo test`.
- Run high-DPI visual comparison subset.
- Run memory profile at supported scale limits.
- Run thumbnail API regression tests.

## Completion Notes

Completed 2026-06-26.

- Added `fixtures/generated/high-dpi-preview-fidelity.pdf` plus
  `fixtures/high-dpi-preview-manifest.tsv` covering high-DPI preview, text,
  vector, image, and transparency baselines.
- Added native regression coverage for scale-aware page cache keys, high-DPI
  visible output, and typed high-DPI raster budget enforcement.
- Documented the existing `max_edge` behavior: it is an output ceiling, not an
  implicit upscale factor for smaller pages.
- Recorded native support, benchmark, batch memory, and visual comparison
  results in
  `docs/reports/high-dpi-thumbnail-preview-fidelity-2026-06-26.md`.

Validation:

- `cargo test -p ferrugo-native high_dpi -- --nocapture`
- `cargo test -p ferrugo-native native_page_cache_key_should_isolate_high_dpi_scale -- --nocapture`
- High-DPI native supported gate, benchmark, batch memory profile, and visual
  comparison subset.
- `cargo test --workspace --no-default-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`
