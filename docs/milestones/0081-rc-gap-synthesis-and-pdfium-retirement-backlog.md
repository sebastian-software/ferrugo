# 0081: RC Gap Synthesis And PDFium Retirement Backlog

Status: done
Phase: 14
Size: small
Depends on: 0080

## Goal

Turn the native renderer release candidate results into a prioritized backlog
that directly reduces PDFium dependence.

## Scope

- Classify every release candidate blocker by document category and renderer
  subsystem.
- Separate correctness, performance, memory, packaging, and API blockers.
- Mark which issues require native renderer work and which only require rollout
  or documentation.
- Define the PDFium fallback removal order for the next milestone wave.

## Non-Goals

- Implement renderer fixes.
- Re-open already accepted unsupported PDF features without evidence.
- Treat pass-rate-only improvements as sufficient for retirement.

## Deliverables

- Gap synthesis report.
- Ordered PDFium retirement backlog.
- Updated support matrix with explicit native blockers.

## Acceptance Criteria

- Each blocker has an owner subsystem and a measurable acceptance gate.
- The next fixes are ordered by real document impact.
- PDFium removal work is tied to evidence from 0080.

## Validation

- Review the 0080 release candidate report.
- Run the corpus summary command used by 0080.
- Verify the milestone backlog has no unscoped placeholder items.

## Completion Notes

Completed on 2026-06-24.

- Reviewed the 0080 RC report and reran the corpus summary command used by the
  gate.
- Published `docs/reports/rc-gap-synthesis-2026-06-24.md`.
- Updated `docs/reports/native-renderer-support-matrix-2026-06-24.md` with the
  0080 generated-corpus blocker state.
- Ordered PDFium retirement around evidence:
  keep PDFium as oracle,
  remove fallback only from reviewed 100% native categories,
  keep fallback for optional content and fidelity-sensitive text,
  defer removal drill until real-world corpus and visual diffing exist.

Primary blockers now have explicit owners and gates:

- optional content / native resource policy,
- visual diff tooling,
- path rasterization performance,
- text fidelity classification,
- real-world corpus ingestion.
