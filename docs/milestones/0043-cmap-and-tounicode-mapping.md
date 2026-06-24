# 0043: CMap And ToUnicode Mapping

Status: done
Phase: 5
Size: medium
Depends on: 0042

## Goal

Interpret common CMap and ToUnicode mappings so text glyph selection and
diagnostics are driven by real PDF font metadata.

## Scope

- Parse simple `ToUnicode` CMaps used by browser and office exports.
- Resolve single-byte encodings and differences arrays where present.
- Carry character-code to glyph-code mappings into text display-list entries.
- Add bounded parsing and explicit unsupported errors for complex CMaps.

## Non-Goals

- Text extraction as a public API.
- Full predefined CMap coverage.
- Vertical writing mode fidelity.

## Deliverables

- CMap parser for the first supported forms.
- Encoding resolution tests.
- Fixture coverage for ToUnicode and differences-array PDFs.

## Acceptance Criteria

- Supported text fixtures map character codes deterministically.
- Unsupported CMap constructs fail without panics or unbounded allocation.
- Diagnostics identify the missing encoding feature.

## Validation

- Run `cargo fmt --check`.
- Run `cargo check`.
- Run `cargo test`.
- Compare text fixture metadata against the PDFium baseline where practical.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Implemented a bounded ToUnicode parser for simple `bfchar` and `bfrange`
  mappings, with byte and entry budgets plus typed unsupported/malformed
  diagnostics.
- Added simple single-byte encoding resolution with Differences-array support
  for common glyph names, and carried source character-code metadata into text
  display-list items.
- Added deterministic `tounicode-text.pdf` and `encoding-differences.pdf`
  fixtures plus native backend smoke coverage.
- Validation: `cargo fmt --check`, `cargo check`, `cargo test`, native CLI
  render smoke for both 0043 fixtures,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `git diff --check`.
