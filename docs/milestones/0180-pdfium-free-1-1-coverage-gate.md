# 0180: PDFium-Free 1.1 Coverage Gate

Status: done
Phase: 33
Size: medium
Depends on: 0179

## Goal

Make the next PDFium-free release decision using expanded typical-document
coverage, native-only validation, and explicit unsupported boundaries.

## Scope

- Run the full native-only validation matrix across the expanded corpus.
- Compare 1.1 coverage, performance, memory, and unsupported categories against
  the 1.0 readiness baseline.
- Decide whether the renderer is ready for 1.1 release, stabilization, or a
  targeted deferral.
- Produce the next implementation backlog from measured gaps.

## Non-Goals

- Claim complete PDF specification support.
- Reintroduce PDFium runtime fallback to pass the gate.
- Ignore documented unsupported cases that affect typical documents.

## Deliverables

- PDFium-free 1.1 coverage report.
- Release, stabilize, or defer recommendation.
- Ranked post-1.1 backlog.

## Acceptance Criteria

- Expanded typical-document families pass native-only gates at documented
  thresholds.
- Performance and memory budgets are measured and acceptable for supported
  workflows.
- Unsupported boundaries remain typed, documented, and visible to consumers.

## Validation

- Run native-only `cargo check`.
- Run native-only `cargo test`.
- Run full supported corpus gate.
- Run visual comparison with the selected PDFium-free oracle strategy.
- Run benchmark and memory profiles.
- Run package dry-runs.
- Run security and fuzz smoke suite.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.

## Completion Notes

Completed on 2026-06-26.

Produced `docs/reports/pdfium-free-1-1-coverage-2026-06-26.md` with the 1.1
release recommendation:

- stabilize the PDFium-free server/runtime path;
- defer a broad PDFium-free 1.1 replacement claim.

Key evidence:

- native-only release gate passed through
  `bash scripts/check_native_only_release.sh`;
- fuzz smoke passed through `bash scripts/check_fuzz_smoke.sh`;
- dashboard run covered support, operator coverage, performance, batch
  isolation, and local corpus metadata;
- strict expanded core supported gate failed on one typed
  `text.font-program` office-export fallback;
- PDFium maintainer visual-diff for the expanded core reported 85 blockers and
  one native error across 98 fixtures.

Ranked post-1.1 backlog:

1. Restore a fallback-free core gate by addressing or explicitly moving the
   `text.font-program` office-export boundary.
2. Reduce office-export `rendering-core` and `text-fonts` visual blockers.
3. Reduce form/widget visual blockers before broad form-facing claims.
4. Keep advanced image filters, dynamic XFA, optional content, color
   management, pattern/mesh shading, and transparency as typed unsupported
   boundaries until implementation milestones land.
