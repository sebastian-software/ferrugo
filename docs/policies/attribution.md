# Attribution Policy

Status: accepted.
Date: 2026-06-24.

`pdfrust` is licensed under either MIT or Apache-2.0 at the user's option.
Project code should remain compatible with that dual-license intent unless a
future decision document explicitly changes the policy.

## Reference Categories

- Behavioral reference: external output, test cases, public documentation, or
  manually observed behavior used to decide how `pdfrust` should behave.
- Architecture inspiration: broad design patterns learned from another project
  without copying protected implementation details.
- Code porting: translating or adapting implementation logic, algorithms, or
  source structure from another project into this repository.

PDFium is the primary behavioral reference for Phase 0 thumbnail output. MuPDF,
Poppler, Ghostscript, PDF.js, `pdf-rs/pdf`, and `lopdf` may be used for
comparison, compatibility research, and architecture-level context.

## Rules

- Do not copy source from AGPL projects into the core library.
- Do not vendor PDFium source into this repository for Phase 0.
- Keep behavior-reference notes in docs, tests, or baseline metadata.
- Direct code porting requires a focused review of the source license, an
  attribution note next to the affected module or document, and a follow-up
  decision record when the port materially shapes project architecture.
- Unsafe or FFI-heavy code should identify the external ABI or documentation it
  follows, but should still expose Rust-native public APIs.

## Phase 0 Defaults

Phase 0 may compare `pdfrust` output to PDFium output and record PDFium build
metadata. It must not commit PDFium source, MuPDF source, Poppler source, or
private corpus PDFs.
