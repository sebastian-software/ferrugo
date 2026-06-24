# 0033: Text State And Font Stubs

Status: done
Phase: 2
Size: medium
Depends on: 0032

## Goal

Interpret basic text operators into positioned text runs, using font stubs
where full font rendering is not ready.

## Scope

- Interpret text object operators such as `BT`, `ET`, `Tf`, `Td`, `Tm`, `Tj`,
  and `TJ`.
- Resolve font resource names to lightweight font descriptors.
- Store positioned text runs in the display list.
- Define explicit unsupported behavior for complex encodings.

## Non-Goals

- Rasterize glyph outlines.
- Implement CMaps or embedded font shaping.
- Extract searchable text as a product API.

## Deliverables

- Text-state interpreter.
- Font descriptor stubs.
- Display-list text runs.

## Acceptance Criteria

- Simple generated text PDFs produce positioned display-list text runs.
- Missing fonts and unsupported encodings return typed unsupported errors.
- Text-state transitions are covered by unit tests.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare text fixture metadata against the PDFium baseline where practical.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed with the `feat: capture positioned text runs` change.

- Added `TextDisplayItem`, `FontDescriptor`, `FontResources`, and
  `build_text_display_list` to `pdfrust-render`.
- Implemented text-state handling for `BT`, `ET`, `Tf`, `Td`, `Tm`, `Tj`, and
  `TJ`.
- Added lightweight ASCII literal-string decoding with explicit unsupported
  behavior for hex strings and non-ASCII text until CMap work lands.
- Added typed errors for missing fonts, text outside `BT`/`ET`, nested text
  objects, unselected fonts, unsupported encodings, and text-run size limits.
- Added tests for the generated `text-page.pdf` fixture, `Tm` plus graphics
  transforms, `TJ` arrays, missing fonts, missing `Tf`, text outside objects,
  unsupported hex text, and text-run limits.
- Validation:
  - `cargo fmt --check`
  - `cargo test -p pdfrust-render`
