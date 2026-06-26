# 0160: PDFium-Free 1.0 Readiness Gate

Status: done
Phase: 29
Size: medium
Depends on: 0159

## Goal

Make the release decision for a PDFium-free Rust-native renderer that covers a
large share of typical documents with explicit unsupported boundaries.

## Scope

- Run the full native-only validation matrix.
- Summarize document-family coverage, known unsupported categories, and risks.
- Verify packaging, API, memory, security, and performance evidence.
- Produce the 1.0 release, stabilization, or defer recommendation.

## Non-Goals

- Claim complete PDF specification support.
- Hide known unsupported cases behind broad marketing language.
- Reintroduce PDFium as a runtime dependency to pass the gate.

## Deliverables

- PDFium-free 1.0 readiness report.
- Release/defer decision with evidence.
- Final blocker or stabilization backlog.

## Acceptance Criteria

- Supported document families pass native-only gates with documented thresholds.
- Runtime PDFium dependency remains absent.
- Remaining unsupported behavior is explicit, typed, and acceptable for the
  release decision.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus visual comparison.
- Run benchmark and memory profiles.
- Run package dry-runs.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

- Added `docs/reports/pdfium-free-1-0-readiness-2026-06-26.md` with the
  PDFium-free 1.0 release decision.
- Decision: defer a broad PDFium-free 1.0 GA / visual replacement claim, but
  approve the PDFium-free supported runtime slice for `browser-print`,
  `office-export`, and `form`.
- Native core gate renders 87/87 fixtures with 0 fallbacks and 0 errors.
- Full corpus renders 177/186 natively; the remaining 8 fallback rows stay
  explicit in typed unsupported buckets, plus 1 encrypted fixture error.
- Core visual PDFium oracle still reports 77 blockers across
  `rendering-core`, `text-fonts`, `annotations-forms`, `page-geometry`, and
  `vector-graphics`, so PDFium remains maintainer oracle tooling.
- Package, plugin-free distribution, PDFium quarantine, native-only
  check/test/clippy, benchmark, batch memory, and fuzz smoke gates passed.
