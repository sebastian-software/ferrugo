# 0001: Rust-First, PDFium-Guided Porting

Date: 2026-06-24.
Status: accepted for planning.

## Context

The project goal is not just to bind PDFium from Rust. The long-term goal is a
Rust-native PDF rendering engine that can generate high-quality thumbnails and,
eventually, broader render output. PDFium remains the most useful reference
because it is permissively licensed, Chrome-adjacent, production-hardened, and
good at real-world PDF edge cases.

The central implementation question is whether to:

1. translate PDFium closely first and clean it up later, or
2. design directly in idiomatic Rust and only port behavior.

## Decision

Use a Rust-first architecture guided by PDFium behavior.

This means:

- Public APIs, ownership, lifetimes, errors, buffers, and module boundaries are
  designed directly in idiomatic Rust.
- PDFium is the behavioral oracle for compatibility, not the shape that the Rust
  code must copy.
- Work proceeds in small vertical slices with differential tests against PDFium.
- The project avoids recreating PDFium's global state, handle-heavy API, and
  single-threaded assumptions unless a narrow subsystem truly requires them.
- Performance is measured from the start, but low-level unsafe optimization is
  only introduced after profiling shows the safe implementation is the problem.

## Rationale

A close C++-to-Rust transliteration would move too much PDFium implementation
shape into the new codebase:

- global initialization patterns,
- opaque handle lifetimes,
- C++ ownership conventions,
- mutation-heavy object graphs,
- historical module boundaries,
- thread-safety constraints that may not be fundamental to the PDF problem.

A completely free greenfield renderer is also too risky. PDF rendering quality
depends on many specification edge cases and malformed real-world documents.
PDFium is valuable because it encodes that accumulated behavior.

The chosen path keeps the Rust codebase clean while still grounding every
behavioral claim in a mature renderer.

## Unsafe Policy

The default rule is safe Rust.

Crates that should start with `#![forbid(unsafe_code)]`:

- parser and syntax,
- object model,
- content stream interpreter,
- high-level thumbnail API,
- error model,
- differential test metadata.

Unsafe code is allowed only in isolated implementation crates or modules where
there is a specific technical reason:

- FFI to PDFium or other native libraries,
- SIMD rasterization,
- tightly audited pixel buffer operations,
- codec integration that cannot be expressed efficiently in safe Rust.

Every unsafe block must document its invariants. Unsafe code must not leak into
the public API as unchecked lifetimes, raw pointers, or caller-managed aliasing.

## Buffer And Copying Policy

Do not use raw pointer copying as the default implementation style.

Prefer safe Rust primitives first:

- `copy_from_slice`,
- `clone_from_slice`,
- slice indexing with checked bounds,
- `chunks_exact` and `chunks_exact_mut`,
- typed pixel buffers,
- explicit stride-aware image rows,
- owned `Vec<u8>` or borrowed slices with clear lifetimes.

`memcpy`-style operations are acceptable when expressed through safe slice
copies. Raw pointer copies are reserved for measured hotspots and must remain
behind small, tested abstractions.

## Performance Policy

Performance matters, especially for thumbnail generation, but it should be
handled with evidence:

- benchmark parse time,
- benchmark first-page render time,
- benchmark thumbnail render time at fixed output sizes,
- track memory high-water mark,
- compare cold and warm runs,
- use profiles before introducing unsafe or SIMD paths.

The first renderer can be safe and simple if it gives the project a correct
test spine. Optimized raster paths can be added behind the same traits later.

## Porting Workflow

Each feature slice should follow this loop:

1. Select a small rendering behavior, such as page size, image placement, path
   fill, simple text, or clipping.
2. Identify the relevant PDFium behavior and fixture expectations.
3. Design the Rust API and data model for that slice idiomatically.
4. Implement the slice in safe Rust where practical.
5. Compare output against PDFium through the differential harness.
6. Profile only after correctness is established.
7. Add unsafe or specialized fast paths only when the profile justifies them.

## Consequences

Positive:

- The Rust codebase can remain idiomatic and auditable.
- The public API is not tied to PDFium's C embedder API.
- Differential tests keep the implementation honest.
- Unsafe code remains rare and reviewable.
- The thumbnail product path can ship on a PDFium backend while the Rust engine
  grows behind the same facade.

Tradeoffs:

- Initial development may be slower than a mechanical transliteration.
- Some PDFium behavior will require investigation instead of direct copying.
- Performance work is deferred until there is enough correctness to measure.
- Compatibility must be built through fixtures and harnesses from the beginning.

## Summary

Safety first in architecture. PDFium parity first in behavior. Performance
second, but measured from the beginning.

