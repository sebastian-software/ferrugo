# Independent Reference Oracle Strategy 2026-06-26

Milestone: 0164

## Decision

Release validation for the supported runtime slice is PDFium-free.

The default server-side path should continue to prove readiness with native-only
fallback gates, native performance/budget gates, packaging checks, and PDFium
quarantine checks. PDFium remains useful as explicit maintainer oracle tooling,
but it is not a normal runtime dependency and is not the release oracle for
supported-family pass/fail decisions.

## Validation Mode Taxonomy

| Mode | Current status | Release role |
| --- | --- | --- |
| Native supported gate | Available through `summarize-fallbacks --fail-on-fallback`. | Primary release gate for supported families. |
| Native budget gate | Available through `benchmark-native`. | Primary release gate for server-side throughput and bounded output. |
| Package/quarantine gate | Available through packaging and quarantine scripts. | Primary release gate for plugin-free/PDFium-free distribution. |
| PDFium visual diff | Available behind `--features pdfium`. | Maintainer triage only. |
| Backend-neutral baseline JSON | Documented in `docs/baselines.md`. | Evidence format, not yet a native-only golden-image gate. |
| Golden image comparison | Not implemented as a committed CLI gate. | Backlog item before it can become release evidence. |
| Multi-oracle review | Strategy defined; provider probes not yet implemented. | Maintainer evidence for disputed behavior. |
| Manual review record | Strategy defined; template/tooling pending. | Fallback for ambiguous visual drift. |

## Corpus And Risk Routing

The supported server-side release path currently routes browser print, office
export, and static forms through native-only gates. Scanner, financial,
government, e-signature, presentation, chart/dashboard, and map fixtures should
also prefer native supported and budget gates where their manifest slice is
classified as supported.

Higher-risk rendering pressure should stay typed rather than silently falling
back to another engine:

| Pressure | Current release treatment |
| --- | --- |
| Optional content | Typed unsupported bucket until native policy and rendering are implemented. |
| Advanced image filters | Typed unsupported bucket until decoder support lands. |
| Pattern/mesh shading | Typed unsupported bucket until native raster behavior lands. |
| Transparency groups and advanced blending | Typed unsupported bucket until compositing support lands. |
| PDF 2.0 black point compensation | Typed color-management unsupported bucket. |
| Dynamic XFA | Typed form unsupported bucket. |

## Historical Visual Evidence

Existing PDFium reports remain archived comparison evidence. They are useful for
prioritizing fidelity work because they expose current visual differences, but
they do not mean production rendering needs PDFium.

The current evidence split is:

- runtime evidence: native-only gates and package/quarantine checks;
- comparison evidence: explicit maintainer visual-diff and metadata commands;
- historical evidence: older PDFium reports and milestone notes.

## Golden Image Gap

`docs/baselines.md` already defines a backend-neutral record format with pixel
hash fields, and `baselines/examples/` contains small JSON examples. There is no
committed native-only `compare-golden` command yet, so no sample golden-image
comparison was run for this milestone. The follow-up backlog records the minimum
work needed before golden images can become a release gate.

## Validation

Native supported gate:

```sh
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family browser-print \
  --include-family office-export \
  --include-family form \
  --fail-on-fallback \
  --max-edge 160 \
  --output target/oracle-0164-native-supported-gate.json
```

Result:

| Total | Native rendered | Fallbacks | Errors |
| ---: | ---: | ---: | ---: |
| 88 | 88 | 0 | 0 |

Family split:

| Family | Total | Native rendered | Fallbacks | Errors |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 11 | 11 | 0 | 0 |
| `form` | 22 | 22 | 0 | 0 |
| `office-export` | 55 | 55 | 0 | 0 |

Golden-image tooling check:

```sh
find baselines -maxdepth 2 -type f -print
rg -n "compare-golden|golden image|golden-image" crates scripts
```

Result: baseline JSON examples exist in `baselines/examples/`, but no committed
golden-image comparison command exists in `crates/` or `scripts/`.

Quarantine/doc split check:

```sh
bash scripts/check_pdfium_quarantine.sh
rg -n "runtime|comparison|historical|PDFium" \
  docs/policies/reference-oracle-strategy.md \
  docs/reports/independent-reference-oracle-strategy-2026-06-26.md \
  docs/policies/visual-diff-thresholds.md
```

Result: `scripts/check_pdfium_quarantine.sh` passed, and the policy/report text
explicitly separates native runtime evidence, maintainer comparison evidence,
and historical PDFium evidence.
