# Native Render Trace Policy

Status: accepted for 0175.
Date: 2026-06-26.

Native render traces are maintainer diagnostics for reducing renderer issues
without relying on PDFium comparison logs. They are opt-in CLI artifacts, not
part of the normal runtime hot path and not a stable public consumer API.

## Trace Format

`pdfrust-cli trace-native` emits JSON with `schema_version: 1` and
`trace_kind: "native-render-trace"`.

The trace records:

- normalized input path for local reproduction;
- page index, maximum render edge, annotation-scan mode, and event limit;
- metadata inspection outcome;
- native render outcome, including dimensions and output byte count;
- aggregate operator coverage;
- bounded operator events derived from compact operator coverage.

The operator event stream is intentionally compact. It preserves operator names,
status, and typed unsupported buckets where available. It does not preserve raw
content stream order beyond the aggregate coverage expansion and is intended for
small reduced fixtures, triage, and regression minimization.

## Privacy Boundary

Traces must not include:

- PDF bytes;
- content stream bytes;
- operands;
- text strings;
- image samples;
- rendered pixel buffers.

Errors may include high-level typed error classes, stable unsupported buckets,
and short diagnostic messages. Maintainers must review new trace fields before
adding them and reject fields that expose document payload data.

The broader diagnostic field classification and sharing checklist lives in
`docs/policies/telemetry-diagnostics-privacy.md`.

## Size Limits

Tracing defaults to 256 operator events and rejects `--max-events` values above
4096. The limit bounds accidental large traces for documents with long content
streams while still allowing reduced fixtures to carry enough detail for issue
triage.

When the source document has more operators than the event limit, traces set:

- `events_emitted` to the bounded count;
- `events_total` to the aggregate operator count;
- `events_truncated` to `true`.

Maintainers should prefer reducing the fixture before increasing the event
limit.

## Replay Boundary

`pdfrust-cli replay-operators` accepts only `native-render-trace` JSON produced
by this tool and emits a compact `operator-replay` JSON summary. Replay counts
bounded operator events and is useful for regression reduction, but it is not a
full PDF interpreter and must not be treated as visual proof.

Visual fidelity still requires native rendering gates, corpus classification,
and visual review where applicable.
