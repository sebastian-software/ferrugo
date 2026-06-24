# Phase 0 Decisions

Status: accepted planning defaults.
Date: 2026-06-24.

Phase 0 is about making the project measurable before implementation choices
become expensive. It does not create the Cargo workspace, Node package,
prebuilt binaries, or a full renderer. It defines the first thumbnail target,
the PDFium probe, and the constraints that later implementation work must
honor.

## Summary

Phase 0 is thumbnail-first:

- Generate preview images, not a general-purpose PDF viewer or editor.
- Build and measure a cut-down PDFium source build first.
- Expose a backend-neutral Rust CLI and library surface first.
- Use RGBA for tests and PNG for user-visible artifacts.
- Keep the first backend serialized.
- Keep npm, Node-API, prebuilt binaries, and distribution policy out of Phase 0.

## 1. Product Target

Options:

- Thumbnail-first.
- Rust-native renderer first.
- PDFium binding product first.

Recommendation:

Start thumbnail-first.

Rationale:

Thumbnail generation is the current product need and gives the project a small,
measurable rendering contract. A full PDFium-class renderer is still the
long-term direction, but it should not define Phase 0.

Consequence:

Phase 0 measures whether a cut-down PDFium backend can solve the immediate
thumbnail need while creating a test spine for a later Rust-native backend.

## 2. License Intent

Options:

- MIT/Apache-2.0 dual license.
- Apache-2.0 only.
- Keep license undecided.

Recommendation:

Use MIT/Apache-2.0 as the planning default.

Rationale:

This fits the Rust ecosystem and npm-friendly distribution better than copyleft
dependencies. It also keeps the project aligned with a permissive long-term
engine goal.

Consequence:

MuPDF and Poppler remain references and comparison renderers, not direct
porting bases. Any direct PDFium code porting requires attribution and license
review. No AGPL dependency should enter the core library.

## 3. First Runtime Surface

Options:

- Rust CLI plus Rust library.
- Node-API first.
- CLI only.

Recommendation:

Start with Rust CLI plus Rust library.

Rationale:

The CLI makes fixture rendering and measurement simple. The library surface
forces an API boundary early without taking on npm, prebuild, or Node lifecycle
complexity.

Consequence:

Node-API remains a planned layer over the Rust library, but it is not a Phase 0
deliverable.

## 4. PDFium Acquisition

Options:

- Build PDFium from source.
- Use a prebuilt PDFium binary.
- Require a manual local PDFium path.

Recommendation:

Build PDFium from source for the Phase 0 probe.

Rationale:

The project needs real measurements for a cut-down configuration. Prebuilt
binaries may render correctly, but they do not answer whether disabling
irrelevant features materially helps size, startup time, or deployment shape.

Consequence:

The probe documents source-build inputs and outputs, but does not commit the
project to shipping PDFium binaries.

Initial GN direction:

```gn
is_debug = false
is_component_build = false
pdf_enable_v8 = false
pdf_enable_xfa = false
pdf_use_skia = false
pdf_use_agg = true
pdf_is_standalone = false
pdf_is_complete_lib = true
clang_use_chrome_plugins = false
use_remoteexec = false
```

## 5. Threading And Isolation

Options:

- Serialized backend.
- Worker thread pool.
- Process isolation.

Recommendation:

Use a serialized PDFium backend in Phase 0.

Rationale:

PDFium's API surface should be treated conservatively around thread safety. A
single render job per backend instance is enough for first measurements and
avoids mixing correctness questions with concurrency questions.

Consequence:

Worker thread pools and process isolation are later design decisions. Phase 0
should still record whether serialization appears operationally acceptable.

## 6. Thumbnail Page Scope

Options:

- Single page with `page_index` defaulting to `0`.
- First page only.
- Batch page rendering.

Recommendation:

Support one page per call, defaulting to page 0.

Rationale:

This covers first-page previews and explicit page thumbnails without creating
batch scheduling, timeout, or memory questions too early.

Consequence:

Batch rendering is a later API extension. The Phase 0 API should not require it.

## 7. Output Formats

Options:

- RGBA and PNG.
- PNG only.
- PNG, JPEG, and WebP.

Recommendation:

Support RGBA and PNG in Phase 0.

Rationale:

RGBA is the best internal comparison format for pixel tests and backend
diffing. PNG is the simplest user-visible artifact for a CLI probe.

Consequence:

JPEG and WebP remain later product-format decisions. Encoder quality settings
should not distract from the PDFium probe.

## 8. Resource Limits

Options:

- Bounded defaults.
- Configurable limits without defaults.
- Best effort only.

Recommendation:

Use bounded defaults.

Default limits:

- `max_edge = 1024` pixels.
- `timeout = 5s` per render job.

Rationale:

Thumbnail generation is likely to run in server or batch contexts. Bounded
defaults make bad PDFs, huge pages, and accidental high-resolution renders less
dangerous from the beginning.

Consequence:

Limits must be overrideable. These are Phase 0 defaults, not final product
limits.

## 9. Error Classes

Options:

- Typed errors from the start.
- Opaque backend errors.
- Exit-code-only CLI errors.

Recommendation:

Use typed errors in the library and map them to clear CLI failures.

Required classes:

- password or encrypted PDF,
- malformed PDF,
- unsupported feature,
- timeout,
- internal error.

Rationale:

The product needs predictable failure behavior at least as much as successful
rendering. Typed errors also make the later Node layer cleaner.

Consequence:

Phase 0 should record how PDFium errors map into these classes, including cases
where the mapping is approximate.

## 10. Fixture Policy

Options:

- Generated fixtures plus a curated local real-world corpus.
- Generated fixtures only.
- Heavy public corpus from the start.

Recommendation:

Use generated fixtures in Git and a curated local real-world corpus outside
Git.

Rationale:

Generated fixtures are license-safe and CI-friendly. Real-world PDFs are still
needed to evaluate invoices, browser PDFs, scans, embedded fonts, vector-heavy
pages, malformed files, and password-protected files.

Consequence:

Large, private, licensed, or user-supplied PDFs stay out of repository history.
The local corpus should be documented by category and metadata, not by committed
file content.

## 11. Distribution

Options:

- No distribution decision in Phase 0.
- Plan npm/prebuilt binaries early.
- Commit to bundling PDFium.

Recommendation:

Make no distribution commitment in Phase 0.

Rationale:

Distribution depends on PDFium build size, platform behavior, licensing review,
and whether the PDFium backend is a temporary bridge or a shipped product path.

Consequence:

Phase 0 must not promise npm packages, prebuilt binaries, or bundled PDFium.
Those decisions happen after the source-build probe and API measurements.

## Phase 0 Exit Criteria

Phase 0 is complete when:

- the cut-down PDFium source build has documented build flags,
- binary size, cold start, render time, thumbnail render time, and memory
  high-water mark are recorded,
- a local CLI can render PNG thumbnails from generated fixtures,
- RGBA output exists for differential comparison,
- the backend-neutral Rust API facade is sketched,
- fixture policy is documented,
- the MIT/Apache-2.0 license intent is documented,
- npm, Node-API, prebuilt binaries, and PDFium bundling are explicitly deferred.

## Deferred Decisions

- npm package naming and package layout.
- Node-API implementation details.
- Prebuilt binary platform matrix.
- Whether PDFium ships as a product backend.
- Worker thread pool or process isolation.
- JPEG and WebP output.
- Batch page rendering.
- Full PDF renderer crate layout.
- Public compatibility with PDFium's C API.

