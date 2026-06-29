# Performance Claims Policy

Status: accepted.
Date: 2026-06-30.

This policy defines when `ferrugo` documentation may make public speed or
memory claims. The goal is to keep README and marketing copy useful without
turning noisy local benchmark slices into broad renderer promises.

## Scope

A performance claim is any statement that says `ferrugo` is faster, smaller,
more memory efficient, comparable to, or slower than another renderer or a prior
Ferrugo release.

Local engineering notes may record raw benchmark observations. Public-facing
README, docs overview, release, package, and website copy must pass the claim
checklist below before adding or strengthening performance language.

## Required Checklist

Every public performance claim needs:

- [ ] Two stable matrix runs.
- [ ] Same host or documented host differences.
- [ ] Reference renderer versions recorded.
- [ ] Timing reliability caveats reviewed.
- [ ] Workload family named.
- [ ] Metric named.
- [ ] Local artifacts named.
- [ ] Claim wording avoids broad renderer parity.

The claim should be phrased by workload family, mode, and metric. For example:
`small-text hot-render p95 improved on this host` is reviewable. `Ferrugo is
faster than PDFium` is not.

## Evidence Rules

- Use release builds for claim evidence.
- Compare the same fixture set, `max_edge`, backend set, warmup, iteration
  count, timeout, and host unless the host difference is part of the claim.
- Treat `benchmark-matrix` `timing_reliability.caveats` as blocking until the
  claim explains why they do not affect the conclusion.
- Memory claims need a named metric: peak RSS, allocation count, allocation
  bytes, retained session bytes, scratch-buffer high-water, or output bytes.
- A 5-10% speed or memory result may be mentioned only when repeated and framed
  as a cumulative track, not as a standalone public win.
- Sub-5% results stay internal unless the claim is explicitly about measurement
  noise or rejected candidates.

## Reference Renderers

PDFium and Poppler are the initial reference renderers. PDFium is required for
in-process hot-render comparison claims when the claim mentions PDFium. Poppler
is a cold-process and visual reference unless a later tool gives it a fair
in-process mode.

MuPDF remains v2 comparison backlog. Do not block current Ferrugo optimization
work on MuPDF setup, and do not make MuPDF comparison claims until the same
matrix fields exist for it.

## CI Policy

The full benchmark matrix remains a local maintainer tool until reference-tool
availability and variance are understood on CI. Focused fixture subsets may
become CI gates only after their variance is measured and their budgets are
documented.

## Update Workflow

Before changing README or other public copy:

1. Run or identify two stable release-mode matrix artifacts.
2. Check `timing_reliability` in both artifacts.
3. Record renderer versions, host details, fixture family, mode, and metric.
4. Write the claim in workload-family terms.
5. Run `bash scripts/check_performance_claims.sh`.
6. Link the supporting plan, report, or benchmark artifact from the change.
