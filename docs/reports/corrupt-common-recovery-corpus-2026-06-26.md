# Corrupt Common Recovery Corpus 2026-06-26

Milestone: 0173

## Summary

Added a focused corrupt-but-common corpus for the Rust-native renderer. The
gate separates local, bounded recovery from corruption that must remain a stable
typed `malformed` error.

The implementation does not broaden parser repair beyond the existing safety
model. It adds fixtures and regression tests for the accepted boundaries:
bounded xref offset drift, malformed linearization hint fallback, benign missing
annotation references, isolated malformed metadata, and hard failures for xref
object mismatch, partial streams, and malformed page trees.

## Fixture Coverage

Added `fixtures/corrupt-recovery-manifest.tsv` with:

| Family | Fixtures | Expected outcome |
| --- | ---: | --- |
| `recoverable-xref-drift` | 1 | Native render through bounded xref offset recovery. |
| `recoverable-linearized-hints` | 1 | Native render through full-loader fallback. |
| `recoverable-broken-annotation` | 1 | Native render while ignoring a missing annotation object. |
| `malformed-metadata` | 1 | Render succeeds; metadata inspection is malformed. |
| `malformed-info-metadata` | 1 | Render succeeds; metadata inspection is malformed. |
| `malformed-xref-mismatch` | 1 | Render and metadata fail as malformed. |
| `malformed-partial-stream` | 1 | Render and metadata fail as malformed. |
| `malformed-page-tree` | 1 | Render and metadata fail as malformed. |

New generated fixtures:

- `fixtures/generated/corrupt-broken-annotation-reference.pdf`
- `fixtures/generated/corrupt-xref-object-mismatch.pdf`
- `fixtures/generated/corrupt-partial-stream.pdf`
- `fixtures/generated/corrupt-page-tree-missing-kids.pdf`
- `fixtures/generated/corrupt-info-metadata-non-dictionary.pdf`

Existing fixtures included in the gate:

- `fixtures/generated/malformed-xref-offset-drift.pdf`
- `fixtures/generated/linearized-malformed-hints.pdf`
- `fixtures/generated/malformed-tagged-structure.pdf`

## Policy Update

Updated `docs/policies/malformed-recovery.md` to make the current boundaries
explicit:

- linearization hint fallback is allowed only when the full document is valid;
- missing annotation references may be ignored for rendering;
- malformed non-visual metadata may fail metadata inspection independently;
- xref object mismatch, partial streams, and missing page-tree structure remain
  non-recoverable.

## Native Coverage

Added:

- `native_backend_should_render_recoverable_corrupt_common_fixtures`
- `native_backend_should_report_common_corruption_as_stable_malformed`
- `native_backend_should_keep_malformed_metadata_from_aborting_render`

## Recoverable Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corrupt-recovery-manifest.tsv \
  --include-family recoverable-xref-drift \
  --include-family recoverable-linearized-hints \
  --include-family recoverable-broken-annotation \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/corrupt-0173-recoverable-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 3 | 3 | 0 | 0 |

## Full Classification

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corrupt-recovery-manifest.tsv \
  --include-family recoverable-xref-drift \
  --include-family recoverable-linearized-hints \
  --include-family recoverable-broken-annotation \
  --include-family malformed-metadata \
  --include-family malformed-info-metadata \
  --include-family malformed-xref-mismatch \
  --include-family malformed-partial-stream \
  --include-family malformed-page-tree \
  --max-edge 160 \
  --output target/corrupt-0173-classification.json
```

Result:

| Total | Native rendered | Fallbacks | Malformed errors |
| ---: | ---: | ---: | ---: |
| 8 | 5 | 0 | 3 |

The two malformed-metadata families render page content successfully and are
covered by direct metadata-inspection tests.

## Benchmark

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/corrupt-recovery-manifest.tsv \
  --include-family recoverable-xref-drift \
  --include-family recoverable-linearized-hints \
  --include-family recoverable-broken-annotation \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/corrupt-0173-benchmark.json
```

Result:

| Family | Total | Native | Fallbacks | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `recoverable-broken-annotation` | 1 | 1 | 0 | 0 | 0 | 5.332 | 5.332 | 38400 |
| `recoverable-linearized-hints` | 1 | 1 | 0 | 0 | 0 | 25.302 | 25.302 | 57600 |
| `recoverable-xref-drift` | 1 | 1 | 0 | 0 | 0 | 3.925 | 3.925 | 38400 |

## Fuzz Smoke

- `cargo run --manifest-path fuzz/Cargo.toml --bin xref_load -- --smoke`
  completed 154 smoke cases.
- `cargo run --manifest-path fuzz/Cargo.toml --bin stream_decode -- --smoke`
  completed 154 smoke cases.
- `cargo run --manifest-path fuzz/Cargo.toml --bin render_setup -- --smoke`
  completed 176 smoke cases.

## Validation

- `python3 scripts/generate_fixtures.py`
- `cargo test -p pdfrust-native corrupt -- --nocapture`
- `cargo test -p pdfrust-native malformed_metadata -- --nocapture`
- `cargo test -p pdfrust-native xref_offset_drift -- --nocapture`
- Recoverable supported gate, full classification, and benchmark commands
  listed above.
- Fuzz smoke commands listed above.
