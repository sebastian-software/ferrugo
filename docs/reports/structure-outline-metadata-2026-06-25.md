# Structure Outline Metadata 2026-06-25

Milestone: 0094.

## Implemented Slice

- Extended backend-neutral `DocumentMetadata` with document info fields,
  structure presence flags, outline metadata, and page labels.
- Added native classic-document extraction for trailer `/Info`, catalog
  `/Metadata`, `/MarkInfo`, `/StructTreeRoot`, named destinations, outlines,
  and direct page labels.
- Kept outline traversal and page-label expansion bounded.
- Extended CLI metadata JSON so corpus extraction includes the new fields.
- Added `metadata-outline-page-labels.pdf`, a deterministic generated fixture
  covering the new metadata surface.

## Support Matrix

| Feature | Native behavior |
| --- | --- |
| Page count and page size | Parsed through the existing page-tree path. |
| Document info | Common `/Info` string/name fields are surfaced. |
| XMP | Presence of catalog `/Metadata` is surfaced. |
| Outlines | Presence and bounded item count are surfaced. |
| Page labels | Direct `/Nums` labels are resolved in page order. |
| Named destinations | Presence through catalog `/Dests` or `/Names /Dests` is surfaced. |
| Tagged PDF | Presence through `/MarkInfo` and `/StructTreeRoot` is surfaced. |

## Fixture Evidence

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated/metadata-outline-page-labels.pdf --manifest fixtures/corpus-manifest.tsv --output target/metadata-0094-fixture.json
```

Extracted metadata:

| Field | Value |
| --- | --- |
| Page count | 1 |
| Page 0 size | `200.000 x 120.000` |
| Title | `Metadata Fixture` |
| Author | `ferrugo` |
| Creator | `fixture generator` |
| Producer | `ferrugo` |
| XMP present | true |
| MarkInfo present | true |
| StructTreeRoot present | true |
| Named destinations present | true |
| Outline items | 2 |
| Page label | `A-1` |

Render smoke:

```text
cargo run -p ferrugo-cli --no-default-features -- render-native fixtures/generated/metadata-outline-page-labels.pdf --max-edge 120 --output target/metadata-0094-fixture.png
```

The metadata fixture also renders natively, confirming the added catalog and
trailer entries do not break the normal thumbnail path.

## Corpus Summary

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/metadata-0094-corpus.json
```

Corpus metadata extraction reported `75` fixtures: `74` successful metadata
records and the existing encrypted fixture error.

Command:

```text
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/metadata-0094-summary.json
```

Fallback summary:

| Total | Native rendered | Fallback required | Errors |
| ---: | ---: | ---: | ---: |
| 75 | 69 | 5 | 1 |

Mixed-layout family:

| Total | Native rendered | Fallback required | Native pass rate | Errors |
| ---: | ---: | ---: | ---: | ---: |
| 15 | 14 | 0 | `0.933` | 1 |

## Validation

```text
python3 scripts/generate_fixtures.py
cargo fmt
cargo fmt --check
cargo check --workspace --no-default-features
cargo test -p ferrugo-native metadata -- --nocapture
cargo test -p ferrugo-cli metadata -- --nocapture
cargo test -p ferrugo-cli comparison_json_should_include_match_status -- --nocapture
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated/metadata-outline-page-labels.pdf --manifest fixtures/corpus-manifest.tsv --output target/metadata-0094-fixture.json
cargo run -p ferrugo-cli --no-default-features -- extract-corpus-metadata fixtures/generated --manifest fixtures/corpus-manifest.tsv --output target/metadata-0094-corpus.json
cargo run -p ferrugo-cli --no-default-features -- render-native fixtures/generated/metadata-outline-page-labels.pdf --max-edge 120 --output target/metadata-0094-fixture.png
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/metadata-0094-summary.json
```

All listed commands completed successfully.

## Remaining Limits

- Extended metadata extraction is currently implemented for classic documents;
  modern/xref-stream inspection still reports page metadata.
- XMP packet parsing, full name-tree traversal, and accessibility role
  extraction remain future work.
- `compare-metadata` continues to compare PDFium parity only for page count and
  page sizes.
