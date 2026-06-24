# pdfrust

`pdfrust` is an exploratory project for PDF thumbnail generation and, longer
term, a Rust-native PDF rendering engine. The starting reference point is
PDFium, the PDF library used by Chromium, with the longer-term goal of making
high-quality PDF rendering available from Rust and Node.js through Node-API
bindings.

This repository currently contains concept and research notes only. It is not
yet an implementation workspace.

Unless noted otherwise, repository code and documentation are licensed under
either MIT or Apache-2.0 at the user's option. PDFium and other renderers are
used as behavior references under the project's attribution policy; their
source code is not vendored here.

## Working Thesis

No open-source, Rust-native renderer appears to match PDFium's rendering
coverage today. The useful Rust ecosystem options are mostly wrappers around
existing C or C++ engines, or libraries focused on parsing, writing, and
manipulating PDFs rather than full-fidelity rendering.

The project should therefore be framed as a long-running engine effort, not a
thin binding project. For short-term product use, binding to PDFium remains the
practical baseline. For the project goal, PDFium should be used as a behavior
oracle while the Rust implementation grows module by module.

The porting philosophy is Rust-first and PDFium-guided: design the architecture,
ownership, buffer handling, and public APIs idiomatically in Rust, while using
PDFium as the compatibility oracle for rendering behavior and edge cases.

Phase 0 is deliberately narrower: build and measure a cut-down PDFium
source-build probe for single-page thumbnail generation, expose a backend-neutral
Rust CLI/library shape, and defer Node-API, npm prebuilds, bundled PDFium, and
full-renderer parity until after those measurements exist.

## Initial Goals

- Generate reliable PDF preview thumbnails as the first measurable product
  target.
- Build a safe Rust core for loading, interpreting, and rendering PDF pages.
- Treat PDFium output as the compatibility baseline for pixel and behavior
  tests.
- Keep the public Rust API idiomatic, with a separate compatibility layer where
  PDFium-like handles or naming are useful.
- Keep the core safe by default; isolate unsafe code to narrow, audited modules
  only when FFI, SIMD, codecs, or measured pixel-buffer hotspots require it.
- Provide a Node.js package via Node-API once the Rust core has a stable render
  surface.
- Design for fuzzing, corpus testing, and malformed-input resilience from the
  beginning.

## Initial Non-Goals

- Shipping an npm package, prebuilt binaries, or bundled PDFium in Phase 0.
- Creating the Cargo workspace, build system, or implementation crates as part
  of the current documentation-only phase.
- A line-by-line rewrite of PDFium as the first milestone.
- Full JavaScript, XFA, signing, editing, and form-filling parity in the MVP.
- Depending on AGPL engines in the core library.
- Exposing PDFium's C API shape as the only native Rust API.

## Documents

- [Rendering landscape](docs/research/2026-06-24-rendering-landscape.md)
- [Porting decision](docs/decisions/0001-rust-first-pdfium-guided-porting.md)
- [Phase 0 decisions](docs/plans/phase-0-decisions.md)
- [Milestones](docs/milestones/README.md)
- [PDFium port strategy](docs/concepts/2026-06-24-pdfium-port-strategy.md)
- [Node API surface](docs/concepts/2026-06-24-node-api-surface.md)
- [Thumbnail generation plan](docs/plans/2026-06-24-thumbnail-generation-plan.md)
- [Roadmap](docs/roadmap.md)
- [Attribution policy](docs/policies/attribution.md)
- [PDFium checkout recipe](docs/build/pdfium-checkout.md)
