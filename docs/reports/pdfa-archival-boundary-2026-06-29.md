# PDF/A Archival Boundary 2026-06-29

Milestone: 0185.

## Summary

Added a bounded native metadata signal for PDF/A-style archival records and a
focused fixture gate for common archive-document thumbnail behavior. The
renderer now surfaces `pdfaid:part`, `pdfaid:conformance`, OutputIntent
presence, and the fact that conformance validation was not performed.

This milestone defines the boundary; it does not add a PDF/A validator.

Policy: `docs/policies/pdfa-archival-boundary.md`

## API And Metadata

`DocumentMetadata` now includes `archival: ArchivalMetadata` with:

| Field | Meaning |
| --- | --- |
| `pdfa_part` | Bounded XMP marker extraction for `pdfaid:part`. |
| `pdfa_conformance` | Bounded XMP marker extraction for `pdfaid:conformance`. |
| `has_output_intents` | Catalog has at least one `/OutputIntents` entry. |
| `conformance_validation_performed` | Always `false`; the renderer does not certify PDF/A. |

The XMP read is limited to 64 KiB of decoded metadata stream bytes.

## Fixture Coverage

Added `fixtures/archival-pdfa-manifest.tsv` with:

| Family | Fixtures | Purpose |
| --- | ---: | --- |
| `pdfa-profile` | 2 | PDF/A-2B-style XMP attributes and PDF/A-3U-style XMP elements with embedded-file context. |
| `embedded-font` | 1 | Existing embedded font rendering baseline for archival records. |
| `output-intent` | 1 | Existing OutputIntent metadata baseline. |
| `metadata` | 1 | Existing XMP, document-info, outline, and page-label metadata baseline. |

The two new generated PDFs are also included in the main corpus manifest under
the `report` family with `pdfa` feature tags.

## Metadata Evidence

Artifact: `target/archival-pdfa-0185-metadata.json`

| Fixture | PDF/A part | Conformance | OutputIntents | Embedded files | Validation performed |
| --- | --- | --- | --- | --- | --- |
| `pdfa-2b-archival-record.pdf` | `2` | `B` | true | false | false |
| `pdfa-3u-embedded-record.pdf` | `3` | `U` | false | true | false |

The second fixture intentionally proves marker extraction and embedded-file
classification without claiming it is a compliant PDF/A-3U file.

## Native Gate Evidence

Artifact: `target/archival-pdfa-0185-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `embedded-font` | 1 | 1 | 0 | 0 |
| `metadata` | 1 | 1 | 0 | 0 |
| `output-intent` | 1 | 1 | 0 | 0 |
| `pdfa-profile` | 2 | 2 | 0 | 0 |
| **Total** | **5** | **5** | **0** | **0** |

## Benchmark Evidence

Artifact: `target/archival-pdfa-0185-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `embedded-font` | 1 | 1 | 1.001 | 1.001 | 0 |
| `metadata` | 1 | 1 | 1.986 | 1.986 | 0 |
| `output-intent` | 1 | 1 | 1.440 | 1.440 | 0 |
| `pdfa-profile` | 2 | 2 | 3.630 | 3.648 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Poppler Visual Evidence

Artifact: `target/archival-pdfa-0185-poppler-visual-diff.json`

Thresholds: default Poppler visual thresholds:
`--max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | Reference errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `pdfa-profile` | 2 | 0 | 2 | 0 | 0 | 0 |

Both fixtures had `p95_channel_delta = 0`; remaining drift is low-amplitude
edge/text rasterization.

## Boundary Notes

- Supported: PDF/A-style metadata marker visibility.
- Supported: native thumbnails for archive records that use supported page
  content.
- Supported: embedded-file presence as inert metadata context.
- Context only: OutputIntent presence.
- Out of scope: PDF/A validation, XMP schema validation, attachment preview,
  color-managed proofing, and legal compliance decisions.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-cli/src/main.rs crates/pdfrust-native/src/lib.rs crates/pdfrust-thumbnail/src/lib.rs fixtures/corpus-manifest.tsv fixtures/archival-pdfa-manifest.tsv scripts/generate_fixtures.py docs/backend/native.md docs/corpus-taxonomy.md docs/milestones/README.md docs/milestones/0185-pdf-a-and-archival-document-conformance-boundary.md docs/policies/pdfa-archival-boundary.md docs/reports/pdfa-archival-boundary-2026-06-29.md
cargo test -p pdfrust-native pdfa -- --nocapture
cargo test -p pdfrust-cli metadata --no-default-features
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/archival-pdfa-manifest.tsv --include-family pdfa-profile --include-family embedded-font --include-family output-intent --include-family metadata --fail-on-fallback --max-edge 160 --output target/archival-pdfa-0185-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/archival-pdfa-manifest.tsv --output target/archival-pdfa-0185-metadata.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/archival-pdfa-manifest.tsv --include-family pdfa-profile --include-family embedded-font --include-family output-intent --include-family metadata --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/archival-pdfa-0185-benchmark.json
cargo run -p pdfrust-cli --no-default-features -- visual-diff-poppler fixtures/generated --manifest fixtures/archival-pdfa-manifest.tsv --include-family pdfa-profile --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --timeout 30 --output target/archival-pdfa-0185-poppler-visual-diff.json
cargo check --workspace --no-default-features
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
