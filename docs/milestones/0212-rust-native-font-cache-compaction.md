# 0212: Rust-Native Font Cache Compaction

Status: done
Phase: 40
Size: medium
Depends on: 0211

## Goal

Reduce memory pressure from repeated font subsets, glyph outlines, CMaps, and
text geometry caches while preserving Rust-native text fidelity.

## Scope

- Profile font cache usage across office, report, browser, CJK, and long-form
  document families.
- Add compaction or eviction for glyph outlines, subset mappings, CMaps, widths,
  and selection geometry where reuse is bounded.
- Validate repeated subset fonts and mixed-script documents under low-memory
  profiles.
- Document cache invariants and diagnostics for memory spikes.

## Non-Goals

- Sacrifice text fidelity for global cache eviction.
- Add process-global mutable caches without clear synchronization boundaries.
- Download or install replacement fonts.

## Deliverables

- Font cache memory profile.
- Compaction or eviction implementation plan and patch set.
- Low-memory text fidelity report.

## Acceptance Criteria

- Repeated font-heavy documents stay within configured memory budgets.
- Font cache eviction does not change rendered output for supported fixtures.
- Diagnostics identify font-cache pressure without exposing document text.

## Validation

- Run `cargo fmt --check`.
- Run native-only `cargo test`.
- Run font subset and CJK corpus comparisons.
- Run low-memory font cache benchmarks.
- Run deterministic render checks before and after cache eviction.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-29.

- Added a resident total decoded embedded-font-program budget per page resource
  map (`max_total_font_program_bytes`) in addition to the existing per-program
  limit.
- Preserved cache sharing for repeated references to the same embedded font
  stream; shared streams count once against the total budget.
- Added typed overflow reporting via `FontProgramResourceBytesOverflow` and
  exposed the budget in native memory diagnostics and CLI diagnostic bundles.
- Added `fixtures/font-cache-compaction-manifest.tsv` covering office, browser,
  report, longform, CJK/CID, subset, and repeated Type3 font families.
- Documented font cache invariants in `docs/backend/native.md` and the baseline
  diagnostics schema in `docs/baselines.md`.
