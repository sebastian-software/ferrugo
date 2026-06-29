# PDFium Port Strategy

Status: exploratory concept note, not an approved implementation spec.
Date: 2026-06-24.

## Position

The project should not begin as a line-by-line rewrite of PDFium. PDFium is a
large C++ engine with decades of accumulated behavior, malformed-file tolerance,
graphics edge cases, codec integration, font behavior, and embedder-facing API
contracts. A direct transliteration would import too much C++ shape into Rust
before the Rust architecture has clear ownership and safety boundaries.

The better first strategy is a compatibility-led Rust implementation:

1. Use PDFium as the behavioral oracle.
2. Build a Rust-native module graph that matches PDF responsibilities, not C++
   file boundaries.
3. Keep an FPDF-like compatibility layer optional and thin.
4. Make pixel tests, corpus tests, fuzzing, and differential tests part of the
   first milestone, not a later cleanup.

This is now captured as a project decision in
`docs/decisions/0001-rust-first-pdfium-guided-porting.md`: safety first in
architecture, PDFium parity first in behavior, and performance second but
measured from the beginning.

## What PDFium Teaches Us

PDFium's public API is centered around initialization, document handles, page
handles, bitmap rendering, text extraction, document editing, forms, and
embedder callbacks. The public headers are the stable embedder surface; internal
code outside `public/` can change at any time.

The internal tree is layered roughly as:

- `core/fpdfapi/parser`: PDF syntax, object graph, cross-reference data, and
  incremental updates.
- `core/fpdfapi/page`: logical page objects, resources, content streams, fonts,
  colorspaces, images, and graphics state.
- `core/fpdfapi/render`: conversion from page objects into drawing operations.
- `core/fpdfdoc`: bookmarks, annotations, links, AcroForms, and higher-level
  document features.
- `core/fpdftext`: text extraction, search, and reading-order logic.
- `core/fxcodec`: PDF stream and image codecs.
- `core/fxge`: graphics engine, raster devices, glyphs, fonts, and platform
  rendering integration.
- `fpdfsdk`: public C API implementation and embedder glue.
- `xfa` and `fxjs`: XFA and JavaScript support when V8/XFA are enabled.

PDFium currently supports AGG as the fully supported graphics backend, Skia as
an experimental backend, optional Fontations work when Skia and Rust support are
enabled, and optional Rust PNG support behind build flags. That is useful
evidence that some parts of the ecosystem are already moving toward Rust-capable
dependencies, but the core engine remains C++.

## Proposed Rust Module Shape

For the later Rust-native engine, start with a Cargo workspace and keep crates
aligned to ownership boundaries. This is not a Phase 0 deliverable; Phase 0 only
needs the backend-neutral thumbnail facade sketched well enough to measure the
PDFium probe.

- `ferrugo-syntax`: byte-level scanner, primitive parser, cross-reference
  parsing, object streams, repair-mode parsing, and source spans for diagnostics.
- `ferrugo-core`: typed object model, documents, pages, resources, name trees,
  metadata, permissions, and high-level document access.
- `ferrugo-filter`: stream filters and image/data codecs. This should isolate
  unsafe or native codec dependencies behind small traits.
- `ferrugo-content`: PDF content stream interpreter, graphics state stack,
  text state, resources, operators, clipping, and display-list emission.
- `ferrugo-font`: font discovery, embedded font parsing, CMaps, glyph mapping,
  text metrics, shaping hooks, and fallback policy.
- `ferrugo-render`: display list, raster device abstraction, color conversion,
  blending, transparency groups, image sampling, antialiasing, and page output.
- `ferrugo-api`: stable Rust API for loading, inspecting, and rendering.
- `ferrugo-capi`: optional C ABI or FPDF-compatible facade if needed for
  integration and differential testing.
- `ferrugo-node`: Node-API bindings and TypeScript package.

## Compatibility Harness

For the Rust-native renderer path, the first real asset should be a harness, not
a renderer. Phase 0 narrows that further to a source-built PDFium thumbnail
probe plus RGBA/PNG output for generated fixtures.

- Build or download a pinned PDFium binary for test runs.
- Render fixtures through PDFium and `ferrugo`, then compare dimensions,
  metadata, errors, text extraction, and pixels.
- Keep separate thresholds for exact pixel equality, antialiasing tolerance, and
  known font/platform drift.
- Track benchmark dimensions explicitly: fidelity, speed, font handling,
  operator coverage, color behavior, and malformed-file robustness.
- Include simple generated PDFs, reduced PDFs based on edge cases, public corpus
  PDFs, fuzz-discovered files, and real-world user documents when licensing
  allows.
- Store expected outputs outside Git when they are large; keep reduced source
  fixtures in Git when license-safe.

## MVP Scope

The MVP should render a constrained but meaningful subset:

- Load PDF 1.7-style files from bytes and file paths.
- Parse classic xref tables, xref streams, object streams, and basic repair
  cases.
- Decode Flate, ASCIIHex, ASCII85, LZW, RunLength, DCT/JPEG, and common image
  predictors.
- Interpret enough content stream operators for paths, fills, strokes, images,
  clipping, simple text, transformations, and graphics state save/restore.
- Render to RGBA bitmaps with deterministic dimensions, scale, rotation, and
  background color.
- Expose page count, page size, metadata, text extraction placeholders, and
  typed errors.

Initial non-MVP items:

- JavaScript execution.
- XFA.
- Digital signatures.
- Full form interaction.
- Full color management and spot colors.
- Editing and incremental save.
- Exact PDFium parity for all malformed files.

## Safety Model

Rust should be used to make invalid states hard to represent:

- Document, page, object, and resource lifetimes should prevent dangling access.
- Parsed objects should distinguish direct values, references, streams, and
  unresolved objects.
- Recursion and stream expansion must be budgeted to avoid decompression bombs
  and pathological object graphs.
- Unsafe code, if required for codecs or SIMD rasterization, must live behind
  tiny audited modules with documented invariants.
- The core should be reentrant by default. Do not recreate PDFium's global
  single-thread assumption unless an isolated subsystem truly requires it.

Parser, object model, content interpretation, and high-level APIs should start
with `#![forbid(unsafe_code)]`. Low-level modules may use unsafe only after the
safe implementation is correct and profiling shows that FFI, SIMD, codec, or
pixel-buffer work requires it.

For copy-heavy rendering code, prefer safe Rust operations such as
`copy_from_slice`, `clone_from_slice`, checked slice indexing,
`chunks_exact_mut`, explicit stride-aware rows, and typed pixel buffers. Raw
pointer copies are an optimization path, not the default design style.

## Porting Workflow

Port behavior in small vertical slices rather than translating whole PDFium
subsystems mechanically.

For each slice:

1. Pick a narrow behavior such as page size, image placement, path fill, simple
   text, clipping, or alpha blending.
2. Identify PDFium's expected behavior with fixtures.
3. Design the Rust data model and API idiomatically.
4. Implement the slice in safe Rust where practical.
5. Compare dimensions, errors, and pixels against PDFium.
6. Profile only after correctness is established.
7. Add unsafe or specialized fast paths only when the profile justifies them.

## Licensing Approach

PDFium uses permissive licensing with BSD-style terms and Apache-2.0 text in the
repository license file. Directly translating implementation files can still
create derivative-work obligations and attribution requirements. Keep porting
records precise:

- Track which PDFium files inform each module.
- Keep copyright and license attribution when behavior or code structure is
  ported.
- Prefer independent Rust design for public APIs and ownership structure.
- Avoid AGPL dependencies in the core library if the intended distribution is
  permissive and npm-friendly.

This is not legal advice; it is an engineering constraint to clarify before the
first source-porting PR.

## When To Stop Or Pivot

Stop the Rust-native engine effort if the actual near-term need is simply
"render PDFs from Node.js reliably." In that case, PDFium bindings or a native
Node binding around PDFium are the practical answer.

Continue if the objective is a long-term Rust engine with memory-safety,
auditable internals, Rust and Node APIs, and an open implementation that can be
tested against PDFium.

## Immediate Next Decisions

- Choose the repository language for public docs: likely English.
- Choose license intent before any code is copied or ported.
- Decide whether the first implementation milestone is parser-first or
  render-harness-first. The recommendation is render-harness-first, because it
  keeps every later milestone measurable against PDFium.
- Decide whether PDFium binaries are downloaded in CI, built from source, or
  provided manually for differential tests.
