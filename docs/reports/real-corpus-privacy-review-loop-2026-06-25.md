# Real Corpus Privacy Review Loop 2026-06-25

Milestone: 0118.

## Implemented Slice

- Added `validate-local-corpus` to `pdfrust-cli` for local-only corpus metadata.
- Replaced the local corpus example with aggregate `[[sample]]` entries that
  record category, privacy, permission, redaction state, coarse page range,
  feature tags, synthetic replacement, and review status.
- Expanded the corpus intake policy and fixture docs with the local validation
  command and the allowed field vocabulary.

No private or third-party PDFs were committed.

## Privacy Gate

The validator accepts only a narrow TOML subset for aggregate local samples. It
rejects private-safety fields such as `path`, `filename`, `hash`,
`text_excerpt`, `screenshot`, and `rendered_output`. CLI output reports only
aggregate counts by category and privacy class.

The committed example validates as:

| Metric | Value |
| --- | --- |
| Sample count | 2 |
| Document count | 5 |
| Categories | invoice 3, scanned-packet 2 |
| Privacy | private 5 |
| Synthetic replacements | 2 |

`fixtures/local-corpus/metadata.toml` is optional for contributors and remains
ignored. With `--allow-missing`, the validator reports a missing local corpus as
a successful empty local gate.

## Synthetic Replacement Check

The shareable replacement path uses `fixtures/real-world-style-manifest.tsv`.
The 0118 run classified 10 production-shaped seed rows and left the other 96
generated fixtures as `unclassified` because the command input was the whole
`fixtures/generated` directory.

Classified seed-category native results:

| Category | Native result |
| --- | --- |
| invoice | rendered |
| statement | rendered |
| scanned-packet | rendered |
| form | rendered |
| browser-export | rendered |
| office-export | rendered |
| report | rendered |
| presentation | fallback required: `graphics.optional-content` |
| secure-document | expected `encrypted` error |
| malformed-recovery | rendered |

Metadata extraction reported 106 generated PDFs total, 10 classified
real-world-style rows, 104 successful metadata inspections, and two expected
metadata errors: one `encrypted` and one `malformed`.

Fallback summary reported 106 generated PDFs total. Across the 10 classified
rows, 8 rendered natively, 1 required the expected optional-content fallback,
and 1 returned the expected encrypted error.

## Validation Commands

```text
cargo fmt --check
cargo check --workspace
cargo test -p pdfrust-cli local_corpus -- --nocapture
cargo run -p pdfrust-cli --no-default-features -- validate-local-corpus fixtures/local-corpus.example.toml
cargo run -p pdfrust-cli --no-default-features -- validate-local-corpus fixtures/local-corpus/metadata.toml --allow-missing
cargo run -p pdfrust-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --output target/real-world-style-0118-metadata.json
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/real-world-style-manifest.tsv --max-edge 160 --output target/real-world-style-0118-fallbacks.json
```

## Limits

- This is not evidence from a real private corpus run.
- Local corpus results should be shared only as aggregate validator and fallback
  summaries.
- Private renderer gaps still need reduced generated fixtures before they are
  useful for open-source regression coverage.
