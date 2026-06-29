# Unsupported Feature Burn-Down 2026-06-29

Milestone 0199 turns the current unsupported-feature surface into explicit
release-candidate decisions. It does not hide unsupported outcomes and does not
route consumers back to runtime PDFium fallback.

## Corpus Result

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --output target/unsupported-0199-classification.json
```

Result:

| Total | Native rendered | Typed unsupported | Malformed errors | Encrypted errors |
| ---: | ---: | ---: | ---: | ---: |
| 227 | 211 | 12 | 3 | 1 |

Unsupported buckets:

| Bucket | Count | Families |
| --- | ---: | --- |
| `image.filter` | 3 | `scan` |
| `graphics.transparency` | 2 | `report` |
| `graphics.optional-content` | 2 | `presentation` |
| `annotation.appearance` | 1 | `mixed-layout` |
| `form.xfa-dynamic` | 1 | `mixed-layout` |
| `graphics.color-management` | 1 | `report` |
| `graphics.pattern-shading` | 1 | `report` |
| `text.font-program` | 1 | `office-export` |

## Fixture-Level Evidence

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --max-edge 160 \
  --iterations 1 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/unsupported-0199-benchmark-fixtures.json
```

Typed unsupported rows:

| Fixture | Family | Bucket | Decision |
| --- | --- | --- | --- |
| `chat-emoji-fallback-boundary.pdf` | `office-export` | `text.font-program` | Release blocker for broad office-export claims; route to 0203 text/font triage. |
| `extgstate-luminosity-soft-mask.pdf` | `report` | `graphics.transparency` | Release blocker for report/dashboard visual-fidelity claims; route to 0213. |
| `freetext-annotation-without-appearance.pdf` | `mixed-layout` | `annotation.appearance` | Documented boundary; route to 0207 annotation fidelity. |
| `mesh-shading-unsupported.pdf` | `report` | `graphics.pattern-shading` | Follow-up vector/chart blocker; route to 0204. |
| `optional-content-ocmd.pdf` | `presentation` | `graphics.optional-content` | Documented OCMD boundary after 0192. |
| `optional-content-usage-application.pdf` | `presentation` | `graphics.optional-content` | Documented usage-application boundary after 0192. |
| `pdf20-black-point-compensation.pdf` | `report` | `graphics.color-management` | Accepted PDF 2.0 deferral unless real-corpus frequency rises. |
| `unsupported-blend-mode.pdf` | `report` | `graphics.transparency` | Release blocker for broad report/dashboard fidelity claims; route to 0213. |
| `unsupported-ccitt-image.pdf` | `scan` | `image.filter` | Release blocker for broad scan/fax/archive claims; route to 0209. |
| `unsupported-jbig2-image.pdf` | `scan` | `image.filter` | Release blocker for broad scan/fax/archive claims; route to 0209. |
| `unsupported-jpx-image.pdf` | `scan` | `image.filter` | Release blocker for broad scan/fax/archive claims; route to 0209. |
| `xfa-dynamic-no-static-appearance.pdf` | `mixed-layout` | `form.xfa-dynamic` | Accepted deferral; dynamic XFA runtime remains out of scope. |

Policy errors are not unsupported buckets:

- malformed recovery fixtures: 3 expected `malformed` errors;
- encrypted placeholder: 1 expected `encrypted` error.

## Strict Supported-Family Gate

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family browser-print \
  --include-family email-web-archive \
  --include-family form \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/unsupported-0199-supported-families.json
```

Result:

| Families | Total | Native rendered | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print`, `email-web-archive`, `form` | 43 | 43 | 0 | 0 |

## 1.2 Readiness Checklist

- Supported strict families: pass.
- Runtime PDFium fallback: remains removed.
- Telemetry/privacy diagnostics: 0198 done.
- Serverless/batch path: 0195 and 0197 done.
- Broad scan/fax/archive claims: blocked by `image.filter`.
- Broad report/dashboard visual-fidelity claims: blocked by
  `graphics.transparency`.
- Broad office-export claims: blocked by `text.font-program`.
- Presentation layers beyond current OCG defaults: documented typed boundary.
- Dynamic XFA: accepted typed boundary.

0200 should use this as the release-decision input rather than treating the
headline 211/227 native-render count as sufficient on its own.

## Validation Commands

```text
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/unsupported-0199-classification.json
cargo run -p ferrugo-cli --no-default-features -- benchmark-native fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --iterations 1 --max-ms 1000 --max-output-bytes 1048576 --output target/unsupported-0199-benchmark-fixtures.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family email-web-archive --include-family form --fail-on-fallback --max-edge 160 --output target/unsupported-0199-supported-families.json
cargo fmt --check
git diff --check -- docs/reports/native-renderer-support-matrix-2026-06-24.md docs/milestones/0199-unsupported-feature-burn-down-release-candidate-gate.md docs/milestones/README.md docs/reports/unsupported-feature-burn-down-2026-06-29.md
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
