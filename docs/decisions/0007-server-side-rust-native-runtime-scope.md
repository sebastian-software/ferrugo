# 0007: Server-Side Rust-Native Runtime Scope

Date: 2026-06-30.
Status: accepted.

## Context

Ferrugo started with a thumbnail-first probe and then grew into a broader
Rust-native PDF renderer effort. During the milestone sweep, the project also
added WASM, low-memory, mobile, serverless, and batch-rendering checks.

Those profiles are useful, but they should not obscure the primary product
shape. The most important near-term use case is server-side document intake:
render bounded previews for untrusted PDFs with explicit time, memory, output,
and feature budgets.

## Decision

Optimize Ferrugo first for server-side Rust-native rendering.

The default runtime path is:

- Rust-native by default;
- bounded by page, pixel, timeout, and memory policies;
- suitable for server and automation workflows;
- explicit about unsupported PDF feature buckets;
- independent from packaged PDFium, Poppler, MuPDF, Ghostscript, or browser
  runtimes.

WASM, embedded, mobile, and low-memory work remains valuable, but it is a
secondary compatibility and packaging profile unless a later product decision
changes the target.

## Rationale

Server-side native rendering gives the project the clearest product and
engineering constraints:

- A compact native binary matters more than browser packaging.
- Hard timeout and process/resource boundaries matter more than UI scheduling.
- Batch and queue behavior matter more than interactive viewer features.
- Typed unsupported outcomes are acceptable when they are stable and
  documented.
- Public performance claims can be tied to concrete host, fixture, and budget
  measurements.

Low-memory discipline still matters. It should be expressed through bounded
session caches, scratch-buffer high-water reporting, and explicit benchmark
fields, not through a WASM-first architecture.

## Consequences

Positive:

- Runtime and release decisions stay focused.
- Performance work can prioritize hot server workloads such as small previews,
  office exports, reports, scans, forms, and batch thumbnail jobs.
- WASM and mobile checks can catch portability problems without blocking the
  main server path.
- Documentation can be more direct about what Ferrugo is useful for today.

Tradeoffs:

- Ferrugo is not currently positioned as a browser-first PDF viewer engine.
- Some WASM-specific optimizations may wait until there is a concrete delivery
  requirement.
- Low-memory profile results are compatibility signals, not the main measure of
  product readiness.

## Follow-Up

- Keep README and release copy centered on server-side native previews.
- Keep WASM and low-memory gates as secondary profile checks.
- Do not add dependencies or abstractions only for browser/WASM delivery unless
  a later ADR changes this scope.
