# Documentation guide

This directory contains the current runtime docs, release notes, policies, and
evidence reports. If you are new to `ferrugo`, start with the user and backend
docs first; use older reports as supporting evidence when a change touches that
subsystem.

## Start here

- [Project README](../README.md): product scope, quick start, and the main
  architectural split.
- [User guide](guides/1-0-user-guide.md): install, CLI rendering, Rust API
  examples, typed errors, troubleshooting, and maintainer-only comparison
  workflows.
- [Rust-native backend](backend/native.md): what the native renderer supports,
  how errors are classified, and where the current limits are.
- [Packaging](packaging.md): native runtime builds, serverless profile,
  plugin-free installation, release automation, and npm packaging.
- [Renderer benchmarks](benchmarks.md): local smoke commands and the current
  performance snapshot for bounded preview workloads.
- [Promoted performance matrix](reports/performance-matrix-promoted-2026-07-04.md):
  the README-backed benchmark snapshot and caveats.
- [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md):
  current follow-up renderer work.

## By task

### I want to render a PDF

Read:

- [User guide](guides/1-0-user-guide.md)
- [Project README](../README.md#quick-start)
- [Packaging](packaging.md#native-only-build)
- [Rust-native backend](backend/native.md#supported-contract)

Useful commands:

```sh
cargo run -p ferrugo --no-default-features -- \
  render fixtures/generated/text-page.pdf \
  --max-edge 256 \
  --output target/text-page.png
```

### I want to understand the renderer architecture

Read:

- [Rust-native backend](backend/native.md)
- [Renderer benchmarks](benchmarks.md)
- [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md)
- [Native renderer API cleanup backlog](backlogs/native-renderer-api-cleanup-backlog.md)

The short version: the public API is Rust-first, PDFium is an external oracle,
and the native renderer is split across parser, object, content, render, font,
image, and document-workflow crates.

### I want to know what the runtime depends on

Read:

- [Packaging](packaging.md)
- [Project README](../README.md#current-status)
- [Attribution policy](policies/attribution.md)

The normal runtime path is Rust-native and does not package external PDF
renderer libraries. Reference-renderer tooling remains available for maintainer
comparison commands.

### I want the current compatibility picture

Read:

- [Rust-native backend](backend/native.md)
- [Unsupported feature SLA](policies/unsupported-feature-sla.md)
- [Consumer migration guide](guides/native-only-consumer-migration.md)
- [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md)

The current runtime path is scoped and tested; visual parity work still has
known gaps. Treat unsupported-feature buckets as renderer backlog, not as a
signal to bundle an external renderer.

### I want to understand performance

Read:

- [Renderer benchmarks](benchmarks.md)
- [Promoted performance matrix](reports/performance-matrix-promoted-2026-07-04.md)
- [Renderer performance optimization backlog](backlogs/renderer-performance-optimization-backlog.md)
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
- [Raster flattening policy](policies/raster-flattening.md)

Backlogs and reports are the working record. Keep validation notes, evidence,
and follow-up owners honest. A passing server gate should not be stretched into
a blanket PDF compatibility claim.

## Main folders

| Folder | What lives there |
| --- | --- |
| `backend/` | Backend-specific behavior and support notes. |
| `build/` | PDFium source-build notes for maintainers. |
| `backlogs/` | Active follow-up work split out from reports, gates, and ADRs. |
| `concepts/` | Earlier design sketches and API concepts. |
| `decisions/` | Architecture decisions that should stay stable. |
| `guides/` | User-facing installation, rendering, migration, and workflow guides. |
| `policies/` | Compatibility, licensing, attribution, API, and support policy. |
| `reports/` | Evidence from gates, benchmarks, corpus sweeps, and release checks. |
| `research/` | Landscape and comparison research. |

## Reading order for maintainers

1. [Project README](../README.md)
2. [User guide](guides/1-0-user-guide.md)
3. [Rust-native backend](backend/native.md)
4. [Packaging](packaging.md)
5. [Renderer benchmarks](benchmarks.md)
6. [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md)
7. The report or policy for the subsystem you are changing

That order gives you the current state before older planning and evidence
reports.
