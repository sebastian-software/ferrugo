# Initial Performance Matrix

Date: 2026-06-29.
Status: implemented as report-first benchmark tooling.

## What changed

`ferrugo-cli benchmark-matrix` now emits one comparison schema for:

- Ferrugo native;
- PDFium, when the `pdfium` feature and `FERRUGO_PDFIUM_LIBRARY` are available;
- Poppler through `pdftoppm`.

The matrix supports two modes:

- `cold-process`: one renderer/tool process per fixture, including startup,
  wall time, exit status, output bytes, output dimensions, and peak RSS when the
  host allows `/usr/bin/time -l`;
- `hot-render`: in-process warmup and measured repetitions for Ferrugo native
  and PDFium, with mean, p50, p95, max, output bytes, and RSS samples.

Poppler is intentionally `not-applicable` for hot-render because it is an
external-process reference in this first slice.

## Fixture families

The starter manifest is `fixtures/performance-matrix-manifest.tsv` and covers:

- `small-text`;
- `office-export`;
- `scan`;
- `browser-print`;
- `form`;
- `presentation`;
- `report/vector`;
- `mixed-layout`.

The `report/vector` family includes known hot-path fixtures such as vector
stress, hatch/clipping, technical linework, and prepress marks.

## Report output

The JSON report includes backend, command, platform, fixture family, mode,
status, timing distribution, output dimensions, output bytes, RSS fields,
error class, and fallback bucket.

The Markdown report lists:

- top 25 slowest Ferrugo fixtures;
- top 25 largest cold-process gaps against the fastest available reference;
- top memory high-water records;
- family summaries with Ferrugo/PDFium and Ferrugo/Poppler ratios where both
  sides rendered.

## Smoke evidence

Command:

```sh
cargo run -p ferrugo-cli --no-default-features -- benchmark-matrix fixtures/generated \
  --manifest fixtures/performance-matrix-manifest.tsv \
  --include-family small-text \
  --include-family scan \
  --backend native \
  --backend pdfium \
  --backend poppler \
  --mode cold-process \
  --mode hot-render \
  --iterations 1 \
  --warmup 0 \
  --max-edge 96 \
  --timeout 20 \
  --output target/performance-matrix-smoke.json \
  --report target/performance-matrix-smoke.md \
  --artifact-dir target/performance-matrix-smoke-artifacts
```

Result: 12 records, 6 rendered, 4 `missing-tool` PDFium records in the
native-only build, 2 Poppler hot-render `not-applicable` records, and 0 errors.

On this sandboxed macOS host, `/usr/bin/time -l` is present but cannot read
`sysctl kern.clockrate`. The harness detects that failure and reruns the
cold-process measurement directly with RSS fields marked as unavailable.

## Optimization loop

Do not optimize from intuition. For the next performance PRs:

1. Run the full matrix on the starter manifest in a release build.
2. Pick the top 5 Ferrugo slow fixtures from the Markdown report.
3. Profile those fixtures with `sample`, Instruments, or Samply.
4. Attribute time to parser/object loading, stream decode, content tokenization,
   display-list construction, resource decode, vector raster, text raster,
   image raster, and output encoding before changing code.
5. Accept changes only with before/after matrix evidence showing at least 10%
   target-fixture improvement or a clear memory win, without new fallback or
   visual regressions.
