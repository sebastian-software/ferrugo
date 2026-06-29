# Fuzzing And Adversarial PDF Hardening

Date: 2026-06-25
Milestone: 0097

## Scope

This milestone added an optional deterministic fuzz-smoke harness and reduced
adversarial regression inputs for native parser and render setup paths.

## Fuzz Smoke Targets

The standalone `fuzz/` package provides these binaries:

| Target | Path |
| --- | --- |
| `primitive_parse` | `parse_primitive`, `parse_primitive_prefix` |
| `xref_load` | indirect object parsing, classic xref loading, modern xref loading |
| `stream_decode` | stream object parsing and bounded stream decode |
| `content_tokenize` | content tokenizer and inline-image tokenization |
| `render_setup` | native metadata inspection and small first-page render setup |

The harness uses deterministic mutations of target-specific seeds, shared PDF
syntax seeds, and committed adversarial corpus files. It intentionally does not
catch panics.

## Hardening Changes

- Added `DEFAULT_MAX_PRIMITIVE_NESTING` to bound nested array/dictionary parsing.
- Added a syntax regression for `fixtures/adversarial/deep-primitive-array.input`.
- Added a content tokenizer regression for
  `fixtures/adversarial/unterminated-inline-image.content`.
- Added a native backend regression for
  `fixtures/adversarial/truncated-header.pdf` to preserve stable `malformed`
  errors for metadata inspection and rendering setup.

## Smoke Results

Commands completed without panics:

```sh
cargo run --manifest-path fuzz/Cargo.toml --bin primitive_parse -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin content_tokenize -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke
cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke
```

Observed smoke case counts:

| Target | Cases |
| --- | ---: |
| `primitive_parse` | 165 |
| `content_tokenize` | 165 |
| `stream_decode` | 154 |
| `xref_load` | 154 |
| `render_setup` | 165 |

## Malformed Corpus Checks

- `cargo test -p ferrugo-syntax excessive_nesting -- --nocapture`
- `cargo test -p ferrugo-content adversarial_unterminated_inline_image -- --nocapture`
- `cargo test -p ferrugo-native adversarial_truncated_header -- --nocapture`

All targeted malformed checks passed.

## Local Instructions

See `docs/fuzzing.md` for smoke commands and corpus update rules.
