# Legal Document Coverage 2026-06-25

Milestone: 0130.

## Decision

Contracts, filing-style pages, visible redactions, scanned legal attachments,
signature-heavy pages, annotations, and form fallback cases now have a focused
native gate. The native renderer renders all 13 legal manifest rows without
PDFium fallback, errors, or benchmark budget failures.

This is visual thumbnail coverage only. The native renderer does not validate
legal signature authenticity, clause semantics, or whether a PDF redaction is
secure. Redaction coverage means visible redaction rectangles are present in
the rendered thumbnail.

## Corpus Additions

New generated fixtures:

| Fixture | Subtype | Coverage |
| --- | --- | --- |
| `legal-contract-signature-blocks.pdf` | contract | clauses, signature stroke, visible reviewed stamp |
| `legal-filing-stamp-comments.pdf` | filing | filing page, highlight band, filed stamp, comment marker |
| `legal-visible-redactions.pdf` | redaction | black redaction rectangles as page content |
| `legal-scanned-attachment-packet.pdf` | scanned attachment | two pages, index page and image-dominant scanned exhibit |

`fixtures/legal-document-manifest.tsv` combines these with existing signature,
highlight annotation, text-note annotation, markup annotation, mobile scan, and
missing-appearance form/annotation baselines.

## Native Gate Evidence

Artifact: `target/legal-0130-supported-gate.json`

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `contract` | 3 | 3 | 0 | 0 |
| `filing` | 3 | 3 | 0 | 0 |
| `redaction` | 2 | 2 | 0 | 0 |
| `scanned-attachment` | 2 | 2 | 0 | 0 |
| `missing-appearance` | 3 | 3 | 0 | 0 |
| **Total** | **13** | **13** | **0** | **0** |

Native tests also verify visible redaction rectangles by sampling black pixels
inside the generated redaction boxes, and render both pages of the scanned
attachment packet through the parallel scheduler.

## Redaction Notes

`legal-visible-redactions.pdf` uses black rectangles as page content. The
thumbnail renderer must preserve those visual rectangles. It does not inspect
hidden text, remove content, validate redaction annotations, or determine
whether the PDF is safely redacted.

Missing-appearance legal rows are included to keep annotation and AcroForm
fallback behavior visible. They may be synthesized, intentionally invisible, or
handled by existing fallback drawing logic, but they must remain typed and
traceable.

## Benchmark Evidence

Artifact: `target/legal-0130-benchmark.json`

| Family | Total | Native rendered | Mean ms | Max ms | Budget failures |
| --- | ---: | ---: | ---: | ---: | ---: |
| `contract` | 3 | 3 | 31.754 | 44.469 | 0 |
| `filing` | 3 | 3 | 15.864 | 45.496 | 0 |
| `redaction` | 2 | 2 | 18.722 | 20.159 | 0 |
| `scanned-attachment` | 2 | 2 | 40.222 | 40.921 | 0 |
| `missing-appearance` | 3 | 3 | 2.858 | 5.577 | 0 |

The benchmark used two iterations, `--max-edge 160`, `--max-ms 1000`, and
`--max-output-bytes 1048576`.

## Visual Oracle Evidence

Artifact: `target/legal-0130-visual-diff.json`

Thresholds: default strict visual review
`--max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05`

| Family | Total | Exact | Accepted drift | Blockers | Native errors | PDFium errors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `contract` | 3 | 0 | 1 | 2 | 0 | 0 |
| `filing` | 3 | 1 | 1 | 1 | 0 | 0 |
| `redaction` | 2 | 0 | 1 | 1 | 0 | 0 |
| `scanned-attachment` | 2 | 0 | 0 | 2 | 0 | 0 |
| `missing-appearance` | 3 | 2 | 0 | 1 | 0 | 0 |
| **Total** | **13** | **3** | **3** | **7** | **0** | **0** |

The blockers are visual-fidelity work, not native coverage failures. They are
concentrated in signature placeholder synthesis, text field synthesis,
contract/filing text and stamp rasterization, scanned image resampling, and
redaction rectangle edge placement.

## Follow-Up Backlog

- Improve form and annotation synthesis parity for legal workflows.
- Add sanitized producer-derived legal filings and contracts after privacy
  review.
- Add explicit redaction annotation fixtures when annotation subtype support is
  expanded.
- Keep semantic redaction and signature validation out of the thumbnail
  renderer unless a future API explicitly scopes those responsibilities.

## Validation Commands

```text
cargo fmt --check
git diff --check -- crates/pdfrust-native/src/lib.rs fixtures/corpus-manifest.tsv fixtures/legal-document-manifest.tsv scripts/generate_fixtures.py
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p pdfrust-native legal -- --nocapture
cargo test --workspace
cargo test --workspace --no-default-features
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/legal-document-manifest.tsv --include-family contract --include-family filing --include-family redaction --include-family scanned-attachment --include-family missing-appearance --fail-on-fallback --max-edge 160 --output target/legal-0130-supported-gate.json
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/legal-document-manifest.tsv --include-family contract --include-family filing --include-family redaction --include-family scanned-attachment --include-family missing-appearance --max-edge 160 --iterations 2 --max-ms 1000 --max-output-bytes 1048576 --output target/legal-0130-benchmark.json
PDFRUST_PDFIUM_LIBRARY=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/pdfrust-tools/pdfium-work/pdfium/out/pdfrust-dylib cargo run -p pdfrust-cli --features pdfium -- visual-diff fixtures/generated --manifest fixtures/legal-document-manifest.tsv --include-family contract --include-family filing --include-family redaction --include-family scanned-attachment --include-family missing-appearance --max-edge 160 --max-mae 2.0 --max-p95 16 --max-changed-ratio 0.05 --output target/legal-0130-visual-diff.json
```
