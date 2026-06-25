# 0213: Transparency Stack Memory Optimization

Status: todo
Phase: 40
Size: medium
Depends on: 0212

## Goal

Optimize Rust-native transparency groups, soft masks, blend isolation, and
temporary raster surfaces so transparency-heavy typical documents stay within
desktop, server, WASM, and low-memory budgets.

## Scope

- Profile transparency stack allocations across presentation, design,
  print-preview, chart, and mixed vector/raster documents.
- Add bounded scratch surfaces, surface reuse, and spill policies where safe.
- Validate soft masks, isolated groups, knockout groups, and blend modes after
  memory optimization.
- Document unsupported or approximated transparency behavior separately from
  memory-budget failures.

## Non-Goals

- Change visual semantics to save memory without explicit unsupported status.
- Optimize unrelated parser or font caches.
- Use PDFium for transparency-heavy fallback rendering.

## Deliverables

- Transparency memory profile.
- Scratch surface and temporary buffer optimization patch set.
- Fidelity and memory regression report.

## Acceptance Criteria

- Transparency-heavy supported documents meet memory budgets.
- Surface reuse does not introduce visual drift beyond documented tolerance.
- OOM, spill, and unsupported cases have typed diagnostics.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run transparency and blend corpus comparisons.
- Run low-memory and WASM transparency benchmarks.
- Run deterministic render checks.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Empty until done.
