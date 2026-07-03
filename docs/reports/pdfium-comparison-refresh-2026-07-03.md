# PDFium Comparison Matrix Refresh

Date: 2026-07-03
Issue: #30

## Summary

The archived 0078 PDFium comparison from 2026-06-24 is superseded for the
focused performance-matrix corpus. Two same-host release-mode matrix runs were
recorded against the pinned PDFium build and both runs rendered all records
without fallbacks, missing tools, not-applicable rows, errors, or timing
reliability caveats.

The results support family-scoped claims only. On this host and corpus,
Ferrugo's native hot-render p95 was lower than PDFium in every family, and
native cold-process wall time was lower or effectively tied with PDFium. Peak
RSS also stayed below PDFium for every family/mode in both runs.

## Environment

- Host: macOS 26.5.1 build 25F80, arm64
- CPU: Apple M1 Ultra, 20 logical CPUs
- Memory: 64 GiB
- Ferrugo: `ferrugo 0.1.0`, release profile
- PDFium revision: `573758fe2dd928279cd52b5a4bc955a6938aab39`
- PDFium GN: `2407 (3357c4f51b1a)`
- PDFium runtime library:
  `/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib`
- PDFium runtime library size: 5.4M

## PDFium Build

The checkout followed `docs/build/pdfium-checkout.md` and used the runtime
dylib args from `docs/build/pdfium-gn-args.md`.

```bash
git checkout 573758fe2dd928279cd52b5a4bc955a6938aab39
/private/tmp/ferrugo-tools/depot_tools/gclient sync --nohooks
/private/tmp/ferrugo-tools/depot_tools/gclient runhooks
/private/tmp/ferrugo-tools/depot_tools/gn gen out/ferrugo-dylib --args='is_debug = false is_component_build = true pdf_enable_v8 = false pdf_enable_xfa = false pdf_use_skia = false pdf_use_agg = true pdf_is_standalone = false pdf_is_complete_lib = false clang_use_chrome_plugins = false use_remoteexec = false treat_warnings_as_errors = false'
ninja -C out/ferrugo-dylib pdfium
```

Smoke probe:

```bash
FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo-pdfium --example smoke
```

Result:

```text
initialized=true last_error=0 library=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib
```

## Matrix Commands

Both matrix runs used the same command shape and were run outside the sandbox so
`/usr/bin/time -l` could collect cold-process peak RSS.

```bash
env FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo --release --features pdfium -- benchmark-matrix fixtures/generated --manifest fixtures/performance-matrix-manifest.tsv --backend native --backend pdfium --mode cold-process --mode hot-render --max-edge 160 --iterations 3 --warmup 1 --timeout 30 --output target/pdfium-matrix-refresh-2026-07-03-run1.json --report target/pdfium-matrix-refresh-2026-07-03-run1.md --artifact-dir target/pdfium-matrix-refresh-2026-07-03-run1-artifacts
env FERRUGO_PDFIUM_LIBRARY=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib/libpdfium.dylib DYLD_LIBRARY_PATH=/private/tmp/ferrugo-tools/pdfium-work/pdfium/out/ferrugo-dylib cargo run -p ferrugo --release --features pdfium -- benchmark-matrix fixtures/generated --manifest fixtures/performance-matrix-manifest.tsv --backend native --backend pdfium --mode cold-process --mode hot-render --max-edge 160 --iterations 3 --warmup 1 --timeout 30 --output target/pdfium-matrix-refresh-2026-07-03-run2.json --report target/pdfium-matrix-refresh-2026-07-03-run2.md --artifact-dir target/pdfium-matrix-refresh-2026-07-03-run2-artifacts
```

## Reliability

| Run | Records | Rendered | Fallbacks | Missing tools | Errors | RSS | Caveats |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Run 1 | 44 | 44 | 0 | 0 | 0 | yes | none |
| Run 2 | 44 | 44 | 0 | 0 | 0 | yes | none |

## Timing Results

Times are milliseconds. Hot-render values are p95. Cold-process values are wall
time. Ratios are Ferrugo/PDFium, so lower is faster for Ferrugo.

| Family | Native hot r1/r2 | PDFium hot r1/r2 | Hot ratio r1/r2 | Native cold r1/r2 | PDFium cold r1/r2 | Cold ratio r1/r2 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `browser-print` | 0.270 / 0.253 | 13.729 / 13.539 | 0.020 / 0.019 | 15.243 / 10.779 | 94.818 / 85.466 | 0.161 / 0.126 |
| `form` | 0.068 / 0.081 | 0.169 / 0.176 | 0.401 / 0.462 | 15.273 / 15.257 | 28.475 / 15.145 | 0.536 / 1.007 |
| `mixed-layout` | 0.264 / 0.153 | 14.305 / 13.857 | 0.018 / 0.011 | 15.224 / 12.158 | 45.100 / 42.428 | 0.338 / 0.287 |
| `office-export` | 0.306 / 0.333 | 13.099 / 13.449 | 0.023 / 0.025 | 15.278 / 15.315 | 43.013 / 45.342 | 0.355 / 0.338 |
| `presentation` | 0.166 / 0.163 | 12.880 / 14.124 | 0.013 / 0.012 | 10.825 / 15.255 | 39.237 / 42.468 | 0.276 / 0.359 |
| `report/vector` | 0.561 / 0.519 | 15.005 / 14.349 | 0.037 / 0.036 | 15.298 / 15.306 | 45.306 / 45.316 | 0.338 / 0.338 |
| `scan` | 0.078 / 0.090 | 0.249 / 0.291 | 0.311 / 0.308 | 15.253 / 15.203 | 15.299 / 26.861 | 0.997 / 0.566 |
| `small-text` | 0.067 / 0.073 | 14.565 / 14.868 | 0.005 / 0.005 | 15.373 / 15.268 | 44.216 / 41.025 | 0.348 / 0.372 |

## Peak RSS Results

Values are maximum peak RSS bytes observed for the family in that mode.

| Family | Native cold r1/r2 | PDFium cold r1/r2 | Native hot r1/r2 | PDFium hot r1/r2 |
| --- | ---: | ---: | ---: | ---: |
| `browser-print` | 3506176 / 3457024 | 24313856 / 24248320 | 4292608 / 4308992 | 27361280 / 27279360 |
| `form` | 2998272 / 2998272 | 20054016 / 20004864 | 3571712 / 3538944 | 23412736 / 23363584 |
| `mixed-layout` | 3325952 / 3325952 | 24215552 / 24248320 | 4685824 / 4505600 | 27394048 / 27312128 |
| `office-export` | 3571712 / 3538944 | 24330240 / 24281088 | 5046272 / 5062656 | 27475968 / 27394048 |
| `presentation` | 3112960 / 3112960 | 24264704 / 24248320 | 5259264 / 5292032 | 27607040 / 27492352 |
| `report/vector` | 3915776 / 3948544 | 24264704 / 24346624 | 6012928 / 5963776 | 27672576 / 27557888 |
| `scan` | 3031040 / 3031040 | 20463616 / 20463616 | 5242880 / 5259264 | 27590656 / 27492352 |
| `small-text` | 2981888 / 2998272 | 24084480 / 24133632 | 5767168 / 5718016 | 27672576 / 27557888 |

## Claim Wording

Policy-compliant wording:

> On the 2026-07-03 macOS arm64 performance-matrix runs, Ferrugo's native
> renderer was below PDFium hot-render p95 in every measured family and below
> PDFium peak RSS in both cold-process and hot-render modes. Cold-process wall
> time was lower than PDFium in all families except the `form` run-2 tie/noise
> case.

Avoid broader wording such as "Ferrugo is faster than PDFium" because this
matrix covers one generated fixture set, one host, one max-edge setting, and
one pinned PDFium revision.

## Follow-up Optimization Issues

No follow-up optimization issue was filed from this refresh. The only
Ferrugo/PDFium cold-process ratio above `1.0` was `form` run 2 at `1.007`, a
sub-1% one-run swing that contradicts run 1 (`0.536`) and falls below the
performance-claims policy threshold for a standalone public claim or regression
issue.
