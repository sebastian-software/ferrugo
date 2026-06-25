# 0209: Rust-Native Image Codec Deployment Policy

Status: todo
Phase: 39
Size: medium
Depends on: 0208

## Goal

Define and validate the Rust-native image codec deployment policy for common
PDF image content across desktop, server, WASM, and low-memory profiles.

## Scope

- Audit JPEG, JPEG 2000, JBIG2, CCITT, Flate, image masks, decode arrays, and
  color-space interactions in the supported corpus.
- Separate built-in Rust decoders, optional native dependencies, and unsupported
  codec boundaries.
- Measure codec memory behavior on large scanned and image-heavy documents.
- Add deployment guidance for profiles that cannot ship specialized codecs.

## Non-Goals

- Accept unsafe decoders without isolation or fuzz evidence.
- Implement every rare image codec before it appears in typical documents.
- Reintroduce PDFium solely for specialized image decoding.

## Deliverables

- Rust-native image codec deployment policy.
- Codec coverage and unsupported-boundary matrix.
- Memory and security report for image-heavy documents.

## Acceptance Criteria

- Common image-heavy documents have a clear native codec path.
- Optional codec dependencies are explicit and profile-specific.
- Unsupported image features produce typed, user-visible diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run image codec corpus comparisons.
- Run scanned-document memory benchmark.
- Run codec fuzz smoke suite.
- Run package profile checks for desktop, server, and WASM.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
