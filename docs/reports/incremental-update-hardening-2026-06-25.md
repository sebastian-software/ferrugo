# Incremental Update Hardening 2026-06-25

Milestone: 0093.

## Implemented Slice

- Added classic xref free-entry tracking for incremental update chains.
- Prevented newer deleted-object entries from resurrecting older object bodies
  with the same object number.
- Kept tombstones keyed by object number instead of full generationed object
  id, because common deleted entries bump the generation while removing an
  older generation.
- Added object-loader coverage for deleted incremental objects.
- Added `incremental-deleted-object.pdf` to the generated corpus and native
  renderer tests.

## Policy

`docs/policies/incremental-and-hybrid-references.md` now states that newer
free xref entries tombstone older in-use entries for the same object number.
This keeps the effective object graph aligned with the latest reachable
revision while avoiding retention of deleted historical objects.

## Fallback Summary

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/incremental-hardening-summary-0093.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 74 | 68 | 5 | 1 |

Mixed-layout family:

| Total | Native rendered | Fallback required | Native pass rate | Errors |
| ---: | ---: | ---: | ---: | ---: |
| 14 | 13 | 0 | `0.929` | 1 |

Fallback categories:

| Feature bucket | Count |
| --- | ---: |
| `image.filter` | 3 |
| `graphics.optional-content` | 1 |
| `graphics.pattern-shading` | 1 |

The remaining mixed-layout error is the existing encrypted placeholder.

## Visual-Diff Run

Command:

```text
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/incremental-hardening-visual-diff-0093.json
```

Corpus summary:

| Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 74 | 27 | 13 | 28 | 5 | 0 | 1 |

Mixed-layout family:

| Total | Exact | Accepted drift | Blockers | Native errors | Both errors |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 14 | 8 | 3 | 2 | 0 | 1 |

Incremental fixture details:

| Fixture | Status | Changed ratio | MAE | P95 delta | Max delta |
| --- | --- | ---: | ---: | ---: | ---: |
| `incremental-deleted-object.pdf` | exact | `0.000000` | `0.000` | `0` | `0` |
| `incremental-update.pdf` | exact | `0.000000` | `0.000` | `0` | `0` |

## Benchmark Run

Command:

```text
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/incremental-hardening-benchmark-0093.json
```

Corpus summary:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 74 | 68 | 5 | 1 | 7 |

Mixed-layout family:

| Total | Native rendered | Fallback required | Errors | Budget failures | Mean ms | Max ms | Output bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 14 | 13 | 0 | 1 | 1 | `33.021` | `170.502` | 688640 |

Incremental fixture outcomes:

| Fixture | Outcome | Mean ms | Budget violations |
| --- | --- | ---: | --- |
| `incremental-deleted-object.pdf` | native rendered | `11.533` | none |
| `incremental-update.pdf` | native rendered | `11.140` | none |

## Validation

```text
python3 scripts/generate_fixtures.py
cargo fmt
cargo test -p pdfrust-object incremental -- --nocapture
cargo test -p pdfrust-native incremental -- --nocapture
cargo fmt --check
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

All commands completed successfully.

The repository still has an unstaged `.gitignore` change with trailing
whitespace that predates this slice. `git diff --check` was run against the
0093 touched files and passed.

## Remaining Limits

- Hybrid compressed type-2 xref-stream entries in the classic loader path
  remain unsupported.
- Signature validation and arbitrary damaged-update repair remain out of
  scope.
- More malformed revision-chain diagnostics can still be expanded in future
  parser-hardening milestones.
