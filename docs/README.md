# Documentation guide

This directory has a lot of history. If you are new to `ferrugo`, start with
the current runtime docs and readiness reports before reading the older planning
material.

## Start here

- [Project README](../README.md): product scope, quick start, and the main
  architectural split.
- [Rust-native backend](backend/native.md): what the native renderer supports,
  how errors are classified, and where the current limits are.
- [Packaging](packaging.md): native runtime builds, serverless profile,
  plugin-free installation, and explicit reference-renderer comparison builds.
- [Renderer benchmarks](benchmarks.md): local smoke commands and the current
  performance snapshot for bounded preview workloads.
- [1.4 readiness report](reports/pdfium-free-1-4-readiness-2026-06-29.md):
  the current release decision and the clearest summary of what is ready.
- [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md):
  the current follow-up work after completed readiness and conformance gates.

## By task

### I want to render a PDF

Read:

- [Project README](../README.md#quick-start)
- [Packaging](packaging.md#native-only-build)
- [Rust-native backend](backend/native.md#supported-contract)

Useful commands:

```sh
cargo run -p ferrugo-cli --no-default-features -- \
  render fixtures/generated/text-page.pdf \
  --max-edge 256 \
  --output target/text-page.png
```

### I want to understand the renderer architecture

Read:

- [Rust-native backend](backend/native.md)
- [Rust-first, PDFium-guided decision](decisions/0001-rust-first-pdfium-guided-porting.md)
- [Roadmap](roadmap.md)
- [Phase 0 decisions](plans/phase-0-decisions.md)

The short version: the public API is Rust-first, PDFium is an oracle, and the
native renderer grows through parser, object, content, render, font, image, and
document-workflow slices.

### I want to know what the runtime depends on

Read:

- [Packaging](packaging.md)
- [PDFium comparison tool removal decision](reports/pdfium-comparison-tool-removal-decision-2026-06-29.md)
- [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md)

The normal runtime path is Rust-native and does not package external PDF
renderer libraries. Reference-renderer tooling remains available for maintainer
comparison commands behind explicit Cargo features.

### I want the current compatibility picture

Read:

- [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md)
- [Native renderer 1.3 coverage scorecard](reports/native-renderer-1-3-coverage-scorecard-2026-06-29.md)
- [Unsupported feature SLA](policies/unsupported-feature-sla.md)
- [Consumer migration guide](guides/native-only-consumer-migration.md)

The broad replacement claim is intentionally deferred. The server/runtime path
is scoped and tested; visual parity work still has known gaps.

### I want to understand performance

Read:

- [Renderer benchmarks](benchmarks.md)
- [Performance optimization working plan](plans/2026-06-29-performance-optimization-working-plan.md)
- [Serverless cold start and binary size](reports/serverless-cold-start-and-binary-size-2026-06-29.md)
- [Server batch throughput](reports/server-batch-throughput-2026-06-25.md)
- [Low-memory renderer profile](reports/low-memory-renderer-profile-2026-06-25.md)

The current numbers are strongest for bounded server-side preview workloads:
small thumbnails, controlled worker counts, explicit pixel budgets, and typed
fallback/error reporting.

### I want to add or triage renderer work

Read:

- [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md)
- [Native renderer API cleanup backlog](backlogs/native-renderer-api-cleanup-backlog.md)
- [Fixture policy](fixtures.md)
- [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md)

Backlogs and reports are the working record. Keep validation notes, evidence,
and follow-up owners honest. A passing server gate should not be stretched into
a blanket PDF compatibility claim.

## Main folders

| Folder | What lives there |
| --- | --- |
| `backend/` | Backend-specific behavior and support notes. |
| `build/` | PDFium source-build notes for maintainers. |
| `concepts/` | Earlier design sketches and API concepts. |
| `decisions/` | Architecture decisions that should stay stable. |
| `guides/` | User-facing migration and workflow guides. |
| `plans/` | Planning baselines and scoped implementation plans. |
| `policies/` | Compatibility, licensing, attribution, API, and support policy. |
| `reports/` | Evidence from gates, benchmarks, corpus sweeps, and release checks. |
| `research/` | Landscape and comparison research. |

## Reading order for maintainers

1. [Project README](../README.md)
2. [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md)
3. [Rust-native backend](backend/native.md)
4. [Packaging](packaging.md)
5. [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md)
6. The report or policy for the subsystem you are changing

That order gives you the current state before the older planning context.
