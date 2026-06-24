# 0033: Text State And Font Stubs

Status: todo
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

Empty until done.
