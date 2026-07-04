# 0010: Phase 0 Product, API, and Runtime Defaults

Date: 2026-07-04.
Status: accepted.

## Context

Phase 0 made Ferrugo measurable before large renderer and packaging decisions
became expensive. The completed planning notes duplicated decisions that now
live in ADRs, policies, reports, and the implementation itself.

ADR 0009 says completed milestone plans should leave the active documentation
path once their durable decisions and evidence have better homes. This ADR
promotes the remaining durable Phase 0 product, API, and runtime defaults out of
the stale planning files.

## Decision

Ferrugo started, and remains scoped, as a thumbnail and preview engine before it
is a general-purpose PDF viewer or editor.

The Phase 0 defaults are:

- target preview images, not full interactive viewing, editing, signing,
  JavaScript execution, dynamic XFA, or a broad PDFium-compatible public API;
- expose a Rust CLI and Rust library facade before Node-API or npm packaging;
- keep the public rendering API backend-neutral so PDFium never defines the
  product surface;
- render one selected page per call, defaulting to page index `0`;
- bound thumbnail size with `max_edge = 1024` by default;
- bound one thumbnail render with a `5s` timeout by default;
- use raw RGBA for tests and backend comparison, and PNG for user-visible CLI
  artifacts;
- report stable error classes for encrypted, malformed, unsupported, timeout,
  and internal failures;
- keep generated fixtures in Git and curated real-world corpus material outside
  Git;
- make no Phase 0 commitment to npm packages, prebuilt binaries, bundled
  PDFium, or a full renderer crate layout.

The initial PDFium probe used source-built PDFium with V8 and XFA disabled,
Skia disabled, AGG enabled, and standalone product targets avoided. That probe
was a measurement and behavior-oracle step, not a decision to ship PDFium as a
normal runtime dependency.

## Durable Homes

The completed Phase 0 evidence and follow-up decisions now live here:

- Phase 0 measurements and completed artifacts:
  `docs/reports/phase-0-report.md`;
- PDFium source-build recipe and GN arguments:
  `docs/build/pdfium-checkout.md` and `docs/build/pdfium-gn-args.md`;
- Rust-first implementation strategy:
  `docs/decisions/0001-rust-first-pdfium-guided-porting.md`;
- timeout and process-isolation policy:
  `docs/decisions/0002-timeout-and-process-isolation.md`;
- server-side native runtime scope:
  `docs/decisions/0007-server-side-rust-native-runtime-scope.md`;
- reference renderers as maintainer oracles:
  `docs/decisions/0008-reference-renderers-as-maintainer-oracles.md`;
- licensing and attribution:
  `LICENSE-MIT`, `LICENSE-APACHE`, and `docs/policies/attribution.md`;
- fixture policy:
  `docs/fixtures.md`;
- error taxonomy:
  `docs/errors.md`;
- baseline metadata:
  `docs/baselines.md`;
- native-only packaging:
  `docs/packaging.md`.

## Consequences

The old Phase 0 decisions note and thumbnail generation plan are retired from
the active documentation tree. Historical context remains recoverable through
Git history, while current readers should use the ADRs, policies, reports, and
backlogs listed above.

Future user-facing documentation should not point readers at Phase 0 planning
notes as current truth. It should point at the current README, native backend
docs, packaging guide, release/readiness reports, and consumer guides.
