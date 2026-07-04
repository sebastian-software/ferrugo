# Reference Oracle Tooling Backlog

Status: partially complete.
Date: 2026-07-04.

This backlog tracks work needed to move visual validation from live PDFium
comparison toward bounded, PDFium-free release evidence.

| Item | Why | Acceptance gate |
| --- | --- | --- |
| Golden image schema extension | Done in `fixtures/native-golden-manifest.tsv` and `docs/policies/native-golden-images.md`. | A committed schema identifies fixture, backend, platform, decoded pixel hash, encoded artifact hash, review owner, and tolerance policy. |
| `compare-golden` CLI command | Done in `ferrugo compare-golden` and `scripts/check_native_golden_images.sh`. | Command renders with no default features and fails on pixel hash or tolerance violations for a bounded manifest. |
| Golden artifact retention policy | Done in `docs/policies/native-golden-images.md`. | Policy caps dimensions, bytes, fixture count, and update process; larger artifacts stay local. |
| Manual review record format | Ambiguous visual drift should be explicit, reviewable evidence. | Markdown or JSON record template captures fixture, family, outputs, decision, reviewer, and follow-up owner. |
| Multi-oracle provider probes | PDFium should not be the only external comparison source for disputed behavior. | Local scripts can record Poppler, MuPDF, PDF.js, or Ghostscript outputs as backend-neutral baseline JSON without adding runtime dependencies. |
| Threshold calibration report | Threshold changes need evidence that they do not mask native regressions. | Report template records before/after counts, subsystem, family, and decision source. |
| CI sample golden set | Done in `fixtures/native-golden-manifest.tsv`. | A tiny manifest covers browser print, office export, static form, scanner, and PDF 2.0 accepted basics with native-only CI. |
| Review dashboard export | Maintainers need to sort visual blockers without keeping unbounded image artifacts. | CLI exports a compact HTML or JSON summary with links to local artifacts and no committed binary churn. |

Until these items land, supported-family release validation remains native
fallback/budget/package based. PDFium visual diff stays maintainer-only triage.

Issue 116 retired the remaining in-process PDFium binding commands. Future
items in this backlog should build on backend-neutral golden records and
external-process oracles.
