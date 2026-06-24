# Rendering Landscape, 2026-06-24

This is a first-pass ecosystem scan for a Rust-native alternative to PDFium.
The conclusion is deliberately conservative: there are useful libraries and
bindings, but no obvious Rust-native, open-source renderer with PDFium-class
coverage and maturity.

## Verdict

Do not stop the project on the assumption that an equivalent Rust-native
renderer already exists. The closest production-grade options are still
existing native engines exposed through bindings:

- PDFium through Rust bindings.
- MuPDF through Rust, JavaScript, or other bindings.
- Poppler through C++/GLib/Qt bindings.

These may be good compatibility baselines or short-term product dependencies,
but they do not satisfy the goal of porting the engine class itself to Rust.

An external comparison by Minal Goel also frames the practical problem well:
different PDF engines can render the same file differently because they make
different choices around PDF operators, font rasterization, color management,
and edge-case handling. That reinforces that this project needs differential
tests and explicit compatibility targets, not just API coverage.

## Existing Options

### PDFium

PDFium remains the most relevant target for this project because it combines a
permissive license profile, Chromium usage, a stable public embedder API, and a
large rendering/test culture. It is also the engine most directly aligned with a
future Node package that wants Chrome-like output without embedding a browser.

The main downside is project size and complexity: PDFium is not just a renderer.
It includes parser, document model, codecs, graphics engine, forms, JavaScript
integration, XFA support, editing APIs, tests, fuzzing, and platform-specific
font/rendering behavior. Porting it to Rust needs staged compatibility, not a
single rewrite pass.

### pdfium-render

`pdfium-render` is the most relevant Rust package if the immediate goal is
"use PDFium from Rust." Its documentation describes it as an idiomatic
high-level Rust interface to PDFium, and PDFium can render pages to bitmaps,
extract text and images, edit files, and create new PDFs.

This is not a Rust port. The crate binds to a PDFium library, usually through a
prebuilt dynamic library or a separately built static archive. The docs also
carry an important operational warning: PDFium itself should be assumed not to
be thread-safe, and `pdfium-render` serializes calls behind a mutex for safety.

Use as: a benchmark, API reference, and possible short-term fallback. Do not use
as evidence that the Rust-native project already exists.

### MuPDF

MuPDF is a high-quality PDF engine written in C. It is fast, embeddable, and
offers broad document functionality, including rendering, extraction, editing,
conversion, redaction, annotation, signing, and more. There is also a Rust
`mupdf` crate, but it depends on `mupdf-sys`; it is a binding, not a port.

The open-source license is AGPL-3.0 for the Rust crate and open-source MuPDF
releases, with commercial licensing available from Artifex. That license profile
is a major constraint for a permissively licensed Node/Rust library.

Use as: a technical comparison point. Avoid depending on it in the core unless
the intended license model changes.

### Poppler

Poppler is a mature PDF rendering library based on Xpdf. The official project
describes it as a PDF rendering library and documents C++, GLib, Qt5, and Qt6
frontends. The current stable release listed by the project on 2026-06-24 is
26.06.0, released on 2026-06-02.

Poppler is relevant as an independent rendering baseline, especially on Linux
and desktop viewers. It is not Rust-native, and its license/profile makes it a
less direct fit for a permissive Node package.

Use as: an additional oracle for rendering disagreements where PDFium behavior
is not obviously correct.

### pdf-rs/pdf

`pdf-rs/pdf` is a Rust library to read, alter, and write PDF files. Its README
points to a separate renderer/viewer via Pathfinder, but the core crate is not
positioned as a full production renderer. It is valuable because it is
Rust-native and PDF-focused, but it does not remove the need for a rendering
engine.

Use as: a reference for Rust PDF object modeling and parser tradeoffs.

### lopdf

`lopdf` is a Rust library for PDF document manipulation. It has strong utility
for reading, modifying, writing, object streams, and some decryption scenarios,
but it is not a rendering engine.

Use as: a reference for low-level PDF structure handling and manipulation, not
as a renderer replacement.

### PDF.js And Ghostscript

PDF.js and Ghostscript should be treated as comparison points rather than direct
implementation bases. PDF.js is valuable because it exposes many real-world web
viewer tradeoffs, but a JavaScript engine is not the target architecture.
Ghostscript is important for print/PDF workflows and conformance history, but it
does not answer the Rust-native/browser-adjacent Node package goal.

Use as: additional baselines for disputed rendering behavior, especially when
PDFium, Poppler, and MuPDF disagree.

## Evaluation Dimensions

The external article usefully groups renderer comparison around fidelity, speed,
and font handling. For this project, expand that into a standing benchmark
matrix:

- Fidelity: pixel output against PDFium, with explicit tolerances for
  antialiasing and platform font drift.
- Speed: parse time, first-page render time, full-document render time, and
  memory high-water mark.
- Font handling: embedded fonts, substituted fonts, CMaps, shaping, fallback,
  glyph cache behavior, and text extraction consistency.
- Operator coverage: content stream operators, graphics state, clipping,
  images, shadings, forms, transparency, patterns, and annotations.
- Color: DeviceRGB/Gray/CMYK first, then ICC-based color, overprint, spot color,
  and blending behavior.
- Robustness: malformed xrefs, damaged streams, recursive resources,
  decompression limits, unsupported security handlers, and timeout/cancellation.

## Implication

The project should separate two paths:

1. A pragmatic compatibility path that can use PDFium itself as a harness,
   oracle, and optional bridge.
2. A Rust-native engine path that ports capabilities in a staged order:
   parser, object graph, filters, content streams, graphics state, fonts,
   images, color, transparency, rasterization, then forms and advanced features.

## Sources

- PDFium repository: https://pdfium.googlesource.com/pdfium
- PDFium README: https://pdfium.googlesource.com/pdfium/+/refs/heads/main/README.md
- PDFium public API header: https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdfview.h
- `pdfium-render` docs: https://docs.rs/pdfium-render/latest/pdfium_render/
- MuPDF: https://mupdf.com/
- `mupdf` Rust docs: https://docs.rs/mupdf/latest/mupdf/
- Poppler: https://poppler.freedesktop.org/
- PDF rendering engines comparison: https://theproductguy.in/blogs/pdf-rendering-engines/
- `pdf-rs/pdf`: https://github.com/pdf-rs/pdf
- `lopdf`: https://github.com/J-F-Liu/lopdf
