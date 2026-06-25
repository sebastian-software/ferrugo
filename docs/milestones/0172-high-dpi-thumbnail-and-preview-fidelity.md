# 0172: High-DPI Thumbnail And Preview Fidelity

Status: todo
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

Empty until done.
