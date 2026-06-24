# 0002: Research And Porting Baseline

Status: done
Phase: 0
Size: small
Depends on: none

## Goal

Capture the initial research baseline for existing PDF rendering engines and the
project's Rust-first, PDFium-guided porting direction.

## Scope

- Record the rendering landscape.
- Compare PDFium, MuPDF, Poppler, PDF.js, Ghostscript, `pdf-rs/pdf`, and
  `lopdf`.
- Record the porting policy: Rust-first architecture, PDFium-guided behavior,
  safe core by default.

## Non-Goals

- Choose every future dependency.
- Prove full renderer feasibility.
- Start source-level porting.

## Deliverables

- `docs/research/2026-06-24-rendering-landscape.md`.
- `docs/decisions/0001-rust-first-pdfium-guided-porting.md`.
- `docs/concepts/2026-06-24-pdfium-port-strategy.md`.

## Acceptance Criteria

- Existing Rust libraries are classified as renderer, binding, parser, or
  manipulation library.
- PDFium is established as the primary behavior oracle.
- MuPDF and Poppler are documented as references, not direct porting bases.
- Unsafe policy is recorded.

## Validation

- Review the research and decision documents for contradictory engine
  recommendations.

## Completion Notes

Completed in commit `43e95c9` (`docs: establish phase 0 thumbnail plan`).

