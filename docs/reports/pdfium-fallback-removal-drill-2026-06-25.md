# PDFium Fallback Removal Drill

Date: 2026-06-25
Milestone: 0099

## Scope

This drill exercised the Rust-native renderer with PDFium fallback disabled for
supported corpus families and recorded the remaining categories that still need
explicit PDFium maintainer coverage or native backlog work.

## Native-Only Supported Gate

Command:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family browser-print \
  --include-family office-export \
  --include-family form \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/drill-0099-supported-gate.json
```

Result:

| Family | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 5 | 5 | 0 | 0 |
| `office-export` | 14 | 14 | 0 | 0 |
| `form` | 11 | 11 | 0 | 0 |
| **Supported gate total** | **30** | **30** | **0** | **0** |

The supported-category gate passes without invoking or requiring PDFium.

## Remaining Fallback Risk

Full corpus summary:

```sh
cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --output target/drill-0099-all-families.json
```

| Family | Total | Native rendered | Fallback required | Error classes |
| --- | ---: | ---: | ---: | --- |
| `browser-print` | 5 | 5 | 0 | none |
| `form` | 11 | 11 | 0 | none |
| `mixed-layout` | 15 | 14 | 0 | `encrypted`: 1 |
| `office-export` | 14 | 14 | 0 | none |
| `presentation` | 4 | 3 | 1 | none |
| `report` | 13 | 12 | 1 | none |
| `scan` | 13 | 10 | 3 | none |

Fallback categories:

| Category | Count | Decision |
| --- | ---: | --- |
| `graphics.optional-content` | 1 | Defer deletion until OCMD policy support lands. |
| `graphics.pattern-shading` | 1 | Defer deletion until mesh shading support lands. |
| `image.filter` | 3 | Defer deletion until CCITT, JBIG2, and JPX policy/support changes land. |

`encrypted` remains a native error class rather than a fallback category.

## Fallback Path Decisions

| Path | Decision | Reason |
| --- | --- | --- |
| Production native-only supported families | Delete fallback usage from deployment/CI config | `browser-print`, `office-export`, and `form` pass the native-only supported gate. |
| `render` / `render-auto --allow-pdfium-fallback` | Keep for explicit maintainer and unsupported-category probes | Remaining unsupported categories still need oracle fallback while native support is incomplete. |
| `PDFRUST_ALLOW_PDFIUM_FALLBACK=1` | Defer removal | Useful for local maintainer sweeps; should not be enabled in production native-only gates. |
| `render-pdfium`, `compare-metadata`, `benchmark-pdfium`, `visual-diff` | Keep | These are comparison tools and are explicitly out of scope for deletion. |
| Native-only disabled PDFium commands | Keep | Clear errors prevent accidental PDFium packaging while preserving script diagnosability. |

## Validation

- `cargo test -p pdfrust-cli fallback_summary -- --nocapture`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/drill-0099-supported-gate.json`
- `cargo run -p pdfrust-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/drill-0099-all-families.json`
- `cargo test -p pdfrust-cli --features pdfium`
