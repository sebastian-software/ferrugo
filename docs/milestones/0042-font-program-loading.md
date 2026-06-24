# 0042: Font Program Loading

Status: done
Phase: 5
Size: medium
Depends on: 0041b

## Goal

Load embedded and base font programs needed by common generated, browser, and
office-like PDFs.

## Scope

- Resolve Type1, TrueType, and CFF font descriptors from page resources.
- Extract embedded font streams with bounded memory use.
- Add a small internal font cache keyed by object identity and font subtype.
- Define stable fallback behavior for missing or unsupported fonts.

## Non-Goals

- Full shaping.
- Full CID-keyed font support.
- System font discovery outside a documented fallback path.

## Deliverables

- Font program loader API.
- Font cache with size limits.
- Fixtures for embedded and base font resource resolution.

## Acceptance Criteria

- Simple embedded-font PDFs reach the glyph preparation layer.
- Missing fonts produce typed unsupported or fallback outcomes.
- Repeated font use across pages does not repeatedly decode the same stream.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Run text fixture comparisons against the PDFium baseline where practical.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Implemented bounded font-program loading for Type1, TrueType, and CFF
  streams, including descriptor resolution, typed fallback/unsupported errors,
  and a per-resource-build cache keyed by stream object identity and program
  kind.
- Added deterministic `embedded-font.pdf` fixture coverage and native backend
  smoke coverage for embedded font resources.
- Validation: `cargo fmt --check`, `cargo check`, `cargo test`,
  native CLI render smoke for `fixtures/generated/embedded-font.pdf`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `git diff --check`.
