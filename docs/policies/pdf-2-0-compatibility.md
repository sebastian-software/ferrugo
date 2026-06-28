# PDF 2.0 Compatibility Boundary

Status: accepted.
Date: 2026-06-26.

The Rust-native renderer accepts PDF 2.0 documents when the syntax and render
features map to existing supported PDF 1.x rendering paths. PDF 2.0 version
markers are not a reason to reject a document by themselves.

## Supported Boundary

Native rendering may proceed when a PDF 2.0 file uses:

- a `%PDF-2.0` header and/or catalog `/Version /2.0`;
- standard page trees, page boxes, resources, content streams, fonts, images,
  and graphics operators already supported by the native renderer;
- non-visual catalog metadata such as associated files, as long as thumbnail
  output does not depend on executing or interpreting that metadata.

These cases must remain covered by committed fixtures and native-only gates.

## Unsupported Boundary

PDF 2.0 features that can materially change rendered pixels must be explicit
typed unsupported outcomes until implemented. The first enforced boundary is
external graphics state black point compensation:

| Feature | Bucket | Policy |
| --- | --- | --- |
| `/UseBlackPtComp true` | `graphics.color-management` | Unsupported until color-management semantics and visual thresholds are defined. |

Unsupported PDF 2.0 behavior must not route consumers back to runtime PDFium.
Maintainer PDFium comparisons may be used only as oracle evidence.

## Triage Rules

- Classify PDF 2.0 features by visual impact before implementation breadth.
- Accept metadata-only structures when thumbnail output is unaffected.
- Add a reduced fixture before changing behavior for a PDF 2.0 feature.
- Prefer typed unsupported buckets over silent approximate rendering for
  color, transparency, layer, security, or annotation semantics that affect
  pixels.

## Usage Classification

Milestone 0181 adds `classify-pdf20-usage` as the repeatable corpus gate for
PDF 2.0 roadmap work. The classifier records version evidence, manifest feature
tags, visual-impact policy, native render outcome, and ranked follow-ups without
persisting PDF bytes, rendered pixels, text samples, stream bytes, or operands.

The 1.2 roadmap should use `docs/backlogs/pdf-2-0-feature-priority-backlog.md`
as the source for PDF 2.0 prioritization. Version markers and metadata-only
associated files remain accepted; `/UseBlackPtComp true` remains the current
typed visual boundary under `graphics.color-management`.
