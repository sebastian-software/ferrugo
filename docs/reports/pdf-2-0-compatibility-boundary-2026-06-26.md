# PDF 2.0 Compatibility Boundary 2026-06-26

Milestone: 0162.

## Decision

Accept PDF 2.0 version markers when the document uses render features already
covered by the native renderer. Reject PDF 2.0 render semantics that can change
pixels and are not implemented yet with typed unsupported buckets.

This milestone adds the first PDF 2.0 fixture corpus and policy. It is not a
claim of complete PDF 2.0 support.

## Fixture Corpus

Manifest: `fixtures/pdf20-compatibility-manifest.tsv`

| Fixture | Family | Expected outcome | Boundary |
| --- | --- | --- | --- |
| `pdf20-basic-office.pdf` | `accepted-office` | Native render | `%PDF-2.0` header plus catalog `/Version /2.0` using existing text/vector paths. |
| `pdf20-associated-files.pdf` | `accepted-associated-file` | Native render | Associated-file metadata is accepted because it is non-visual for thumbnail output. |
| `pdf20-black-point-compensation.pdf` | `unsupported-color-management` | Typed unsupported | `/UseBlackPtComp true` affects color-management semantics and maps to `graphics.color-management`. |

The first two fixtures are also in the full corpus as `office-export` and
`mixed-layout`. The black-point compensation boundary is in the full corpus as
`report`.

## Policy Summary

Policy: `docs/policies/pdf-2-0-compatibility.md`

Supported:

- `%PDF-2.0` header and catalog `/Version /2.0`.
- Standard page, resource, text, vector, and image features already supported
  by native rendering.
- Metadata-only associated files that do not affect thumbnail output.

Unsupported:

- `/UseBlackPtComp true` in external graphics state until color-management
  semantics and thresholds are implemented.
- Future PDF 2.0 features that affect color, transparency, layers, security, or
  annotation appearance must get reduced fixtures and typed buckets before
  being approximated.

## Validation

Commands run:

```sh
cargo check --workspace --no-default-features
cargo test -p pdfrust-render black_point -- --nocapture
cargo test -p pdfrust-native pdf20 -- --nocapture
cargo test --workspace --no-default-features
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/pdf20-compatibility-manifest.tsv --include-family accepted-office --include-family accepted-associated-file --fail-on-fallback --max-edge 160 --output target/pdf20-0162-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/pdf20-compatibility-manifest.tsv --include-family accepted-office --include-family accepted-associated-file --include-family unsupported-color-management --max-edge 160 --output target/pdf20-0162-classification.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/pdf20-0162-affected-supported-gate.json
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
