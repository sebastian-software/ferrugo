# Native Render Trace Operator Replay 2026-06-26

Milestone: 0175

## Summary

Added opt-in native maintainer tooling for bounded render traces and compact
operator replay. The tools help reduce and triage native renderer issues without
depending on PDFium comparison logs.

The trace path is CLI-only and does not add hooks to normal render commands.
Tracing reads the target PDF, records metadata/render outcomes, captures
aggregate operator coverage, and emits bounded operator events that omit
document payload data.

## Commands

Added:

- `pdfrust-cli trace-native <input.pdf>`
- `pdfrust-cli replay-operators <trace.json>`

Trace options:

- `--output PATH`
- `--page-index N`
- `--max-edge N`
- `--max-events N`
- `--no-annotations`

`--max-events` defaults to 256 and is capped at 4096.

## Trace And Replay Smoke

Trace command:

```sh
cargo run -p pdfrust-cli --no-default-features -- trace-native fixtures/generated/vector-paths.pdf \
  --max-edge 160 \
  --max-events 8 \
  --output target/trace-0175-vector.json
```

Replay command:

```sh
cargo run -p pdfrust-cli --no-default-features -- replay-operators target/trace-0175-vector.json \
  --output target/replay-0175-vector.json
```

Artifact sizes:

| Artifact | Bytes |
| --- | ---: |
| `target/trace-0175-vector.json` | 2218 |
| `target/replay-0175-vector.json` | 152 |

Trace result:

| Field | Value |
| --- | --- |
| Render status | `rendered` |
| Output size | `160x131` |
| Output bytes | `83840` |
| Streams scanned | `1` |
| Total operators | `11` |
| Events emitted | `8` |
| Events truncated | `true` |

Replay result:

| Field | Value |
| --- | --- |
| Events replayed | `8` |
| Operator counts | `Q:1`, `RG:1`, `S:1`, `f:1`, `l:2`, `m:1`, `q:1` |

## Privacy Review

Reviewed the trace artifact for accidental payload leakage with targeted string
searches for stream markers, fixture text, common content stream fragments, text
operators, and inline-image markers.

The only matches were diagnostic field names and policy text such as
`streams_scanned` and the explicit privacy statement. No PDF bytes, stream
bytes, operands, text strings, image samples, or rendered pixel buffers were
observed in the trace output.

## Disabled Benchmark

Tracing is opt-in. The disabled comparison used the existing native benchmark
command, which does not call the trace path:

```sh
cargo run -p pdfrust-cli --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/corpus-manifest.tsv \
  --include-family report \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output target/trace-0175-disabled-benchmark.json
```

Result:

| Total | Native rendered | Fallback required | Errors | Budget failures |
| ---: | ---: | ---: | ---: | ---: |
| 44 | 40 | 4 | 0 | 4 |

Report-family timing:

| Mean ms | Max ms | Output bytes |
| ---: | ---: | ---: |
| 51.665 | 290.387 | 2556480 |

## Validation

- `cargo test -p pdfrust-cli trace -- --nocapture`
- `cargo test -p pdfrust-cli replay_operator -- --nocapture`
- Trace smoke command on `fixtures/generated/vector-paths.pdf`.
- Replay smoke command on the generated trace.
- Privacy review of the trace output.
- Disabled benchmark on the report corpus family.
