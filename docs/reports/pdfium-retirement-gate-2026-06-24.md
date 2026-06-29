# PDFium Retirement Gate

Date: 2026-06-24.
Milestone: 0060.

## Decision

No-go for PDFium retirement.

PDFium must remain the oracle and production fallback after this phase. The
Rust-native backend is strong enough to keep expanding fixture and local-corpus
coverage, but it is not yet justified as the default renderer for typical
documents and PDFium is not an optional-removal candidate.

## Evidence Summary

The committed generated fixture set was rendered through both backends at
`max-edge 120` using the local debug CLI. The local PDFium build was available
at:

```text
/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib
```

Native results:

- 37 generated fixtures rendered successfully.
- 1 encrypted fixture returned the expected public `encrypted` class.
- 1 valid fixture, `optional-content-ocmd.pdf`, returned public `unsupported`.

PDFium results:

- 38 generated fixtures rendered successfully.
- 1 encrypted fixture returned the expected public `encrypted` class.

Current local-corpus evidence is insufficient for retirement. The repo has a
manifest template at `fixtures/local-corpus.example.toml`, but no committed or
discoverable local corpus manifest is available in the worktree. Real typical
categories therefore remain represented by the support-matrix expectations,
not by a fresh full local-corpus run.

## Generated Fixture Run

Command shape:

```sh
cargo build -p ferrugo-cli
target/debug/ferrugo-cli render-native fixtures/generated/<fixture>.pdf \
  --max-edge 120 \
  --output target/ferrugo-retirement-0060/<fixture>-native.png
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib \
DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib \
target/debug/ferrugo-cli render fixtures/generated/<fixture>.pdf \
  --max-edge 120 \
  --output target/ferrugo-retirement-0060/<fixture>-pdfium.png
```

| Fixture | Native | PDFium |
| --- | --- | --- |
| `acroform-checkbox.pdf` | ok | ok |
| `acroform-signature-placeholder.pdf` | ok | ok |
| `acroform-text-field.pdf` | ok | ok |
| `annotation-appearance.pdf` | ok | ok |
| `annotation-missing-appearance.pdf` | ok | ok |
| `axial-gradient.pdf` | ok | ok |
| `blend-modes.pdf` | ok | ok |
| `clipped-paths.pdf` | ok | ok |
| `cmyk-image.pdf` | ok | ok |
| `dashed-stroke.pdf` | ok | ok |
| `dct-image.pdf` | ok | ok |
| `embedded-font.pdf` | ok | ok |
| `encoding-differences.pdf` | ok | ok |
| `encrypted-placeholder.pdf` | encrypted | encrypted |
| `form-xobject.pdf` | ok | ok |
| `highlight-annotation-appearance.pdf` | ok | ok |
| `hybrid-reference.pdf` | ok | ok |
| `image-xobject.pdf` | ok | ok |
| `incremental-update.pdf` | ok | ok |
| `indexed-image.pdf` | ok | ok |
| `inline-image.pdf` | ok | ok |
| `line-caps.pdf` | ok | ok |
| `line-joins.pdf` | ok | ok |
| `link-annotation-appearance.pdf` | ok | ok |
| `malformed-xref-offset-drift.pdf` | ok | ok |
| `optional-content-layer-off.pdf` | ok | ok |
| `optional-content-layer-on.pdf` | ok | ok |
| `optional-content-ocmd.pdf` | unsupported | ok |
| `page-size-letter.pdf` | ok | ok |
| `predictor-image.pdf` | ok | ok |
| `radial-gradient.pdf` | ok | ok |
| `soft-mask-image.pdf` | ok | ok |
| `text-page.pdf` | ok | ok |
| `text-spacing.pdf` | ok | ok |
| `tiling-pattern.pdf` | ok | ok |
| `tounicode-text.pdf` | ok | ok |
| `transparency-group.pdf` | ok | ok |
| `vector-paths.pdf` | ok | ok |
| `widget-annotation-appearance.pdf` | ok | ok |

## Retirement Threshold

PDFium can move from production fallback to optional dependency only after all
of these are true:

- Native renders all committed generated fixtures except expected `encrypted`
  and intentional malformed-policy fixtures.
- Native has a local-corpus manifest covering office export, browser print,
  invoice, scanned page, image-heavy, vector-heavy, encrypted, and malformed
  categories.
- Native succeeds or returns documented typed fallback reasons for every
  local-corpus document.
- Valid PDFium-success documents do not produce silent blank, materially wrong,
  or unclassified native output.
- Performance and memory measurements stay within the renderer budget policy.

Native can become the default before PDFium is optional only behind a fallback
policy that retries PDFium for public `unsupported` outcomes and records the
fallback reason.

## Blockers

1. `optional-content-ocmd.pdf` is still a valid PDFium-rendered fixture that
   native classifies as `unsupported`.
2. Real local-corpus categories have not been freshly measured in this gate.
3. Text output still includes fallback/degraded paths where fidelity-sensitive
   callers need PDFium as an oracle.
4. PDFium visual comparisons are currently manual render checks; automated
   pixel-diff thresholds are still part of later benchmark and release-candidate
   gates.

## Recommendation

Keep PDFium as fallback and oracle. Continue with the native-default rollout
only as an explicit fallback experiment, not as retirement:

- 0061 should gate native-default behavior behind supported categories and
  preserve PDFium fallback for public `unsupported` outcomes.
- 0062 should add structured fallback telemetry so remaining PDFium usage is
  counted by typed reason.
- 0063 should expand the local corpus before any renewed retirement decision.
