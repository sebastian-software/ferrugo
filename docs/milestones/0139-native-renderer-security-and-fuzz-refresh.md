# 0139: Native Renderer Security And Fuzz Refresh

Status: done
Phase: 25
Size: medium
Depends on: 0138

## Goal

Refresh security, fuzzing, and adversarial-input coverage for the native
renderer after the broader document-family expansion.

## Scope

- Review parser, font, image, and raster budget boundaries.
- Refresh fuzz corpora with minimized failures from recent milestones.
- Add regression tests for panics, excessive allocation, and slow inputs.
- Document remaining untrusted-input assumptions and limits.

## Non-Goals

- Prove memory safety of third-party dependencies.
- Accept unbounded repair of malformed files.
- Treat fuzzing as a replacement for targeted renderer tests.

## Deliverables

- Security and fuzz refresh report.
- Updated minimized adversarial corpus.
- Budget and panic regression tests.

## Acceptance Criteria

- Recent document-family fixtures do not introduce unbounded work paths.
- Known adversarial cases fail with typed, bounded errors.
- Fuzz findings are reduced or explicitly tracked as follow-up issues.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run adversarial fixture corpus.
- Run fuzz smoke or configured local fuzz target.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `fixtures/adversarial/huge-image-dimensions.pdf` and seeded it into
  the `render_setup` fuzz smoke target.
- Added renderer and native backend regressions proving huge declared image
  sample dimensions fail as `renderer.memory-budget` before allocation.
- Moved declared XObject and inline-image sample-size checks ahead of decode
  work while preserving existing image data length validation.
- Refreshed `docs/fuzzing.md`, `docs/backend/native.md`, and the security/fuzz
  report:
  `docs/reports/native-renderer-security-fuzz-refresh-2026-06-26.md`.
- Current fuzz smoke counts:
  `primitive_parse` 165, `xref_load` 154, `stream_decode` 154,
  `content_tokenize` 165, `render_setup` 176.
