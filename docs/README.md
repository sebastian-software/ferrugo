# Documentation guide

This directory has a lot of history. If you are new to `ferrugo`, start with
the current runtime docs and readiness reports before reading the older planning
material.

## Start here

- [Project README](../README.md): product scope, quick start, and the main
  architectural split.
- [Rust-native backend](backend/native.md): what the native renderer supports,
  how errors are classified, and where the current limits are.
- [Packaging](packaging.md): native-only builds, serverless profile,
  plugin-free installation, and explicit PDFium comparison builds.
- [Milestones](milestones/README.md): the implementation log. The files stay in
  place and use `Status:` plus completion notes instead of moving between
  folders.
- [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md):
  the current release decision and the clearest summary of what is ready.

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

### I want to know whether PDFium is still required

Read:

- [Packaging](packaging.md)
- [PDFium comparison tool removal decision](reports/pdfium-comparison-tool-removal-decision-2026-06-29.md)
- [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md)

The normal runtime path does not package PDFium. PDFium remains available for
maintainer comparison commands behind an explicit Cargo feature.

### I want the current compatibility picture

Read:

- [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md)
- [Native renderer 1.3 coverage scorecard](reports/native-renderer-1-3-coverage-scorecard-2026-06-29.md)
- [Unsupported feature SLA](policies/unsupported-feature-sla.md)
- [Consumer migration guide](guides/native-only-consumer-migration.md)

The broad replacement claim is intentionally deferred. The server/runtime path
is scoped and tested; visual parity work still has known gaps.

### I want to add or triage renderer work

Read:

- [Milestones](milestones/README.md)
- [Native renderer conformance backlog](backlogs/native-renderer-conformance-backlog.md)
- [Native renderer API cleanup backlog](backlogs/native-renderer-api-cleanup-backlog.md)
- [Fixture policy](fixtures.md)

Milestone docs are the working record. Keep the status field, validation notes,
and report links honest. A passing server gate should not be stretched into a
blanket PDF compatibility claim.

## Main folders

| Folder | What lives there |
| --- | --- |
| `backend/` | Backend-specific behavior and support notes. |
| `build/` | PDFium source-build notes for maintainers. |
| `concepts/` | Earlier design sketches and API concepts. |
| `decisions/` | Architecture decisions that should stay stable. |
| `guides/` | User-facing migration and workflow guides. |
| `milestones/` | Numbered implementation plan and completion log. |
| `plans/` | Planning baselines and scoped implementation plans. |
| `policies/` | Compatibility, licensing, attribution, API, and support policy. |
| `reports/` | Evidence from gates, benchmarks, corpus sweeps, and release checks. |
| `research/` | Landscape and comparison research. |

## Reading order for maintainers

1. [Project README](../README.md)
2. [PDFium-free 1.4 readiness](reports/pdfium-free-1-4-readiness-2026-06-29.md)
3. [Rust-native backend](backend/native.md)
4. [Packaging](packaging.md)
5. [Milestones](milestones/README.md)
6. The report linked from the milestone or subsystem you are changing

That order gives you the current state before the older planning context.
