# Native Conformance Triage

Status: accepted.
Date: 2026-06-26.

Native renderer conformance triage turns visual oracle output into small,
owner-ready follow-up work. It is not a claim that PDFium is always correct;
PDFium is a maintainer oracle used to expose differences that need a native
decision.

## Report Contract

Every conformance triage report should include:

| Field | Meaning |
| --- | --- |
| `artifact` | Path to the source CLI JSON report. |
| `command` | Exact command used to generate the artifact. |
| `thresholds` | Visual thresholds used for exact, accepted drift, and blocker classification. |
| `summary` | Total, exact, accepted drift, blocker, native-error, PDFium-error, and both-error counts. |
| `families` | Per-family visual status counts. |
| `subsystems` | Per-subsystem visual status counts. |
| `triage_rows` | Blocker clusters grouped by subsystem, family, and recommended next action. |
| `expected_drift` | Drift rows that are acceptable under current thresholds, with rationale. |
| `typed_unsupported` | Native errors that represent explicit unsupported feature boundaries. |
| `next_actions` | Small follow-up slices with a reproducible validation gate. |

The underlying CLI JSON uses `schema_version: 1`. Until a dedicated schema
validator exists, validation means checking that the CLI command exits
successfully, the JSON contains `schema_version: 1`, and the report preserves
the family/subsystem/status counts from the artifact.

## Status Classes

| Status | Triage meaning |
| --- | --- |
| `exact` | Native and oracle output match at the pixel threshold. |
| `accepted_drift` | Difference is below thresholds or already documented as intentional low-amplitude drift. |
| `blocker` | Visual difference exceeds thresholds and needs a native decision or implementation slice. |
| `native_error` | Native renderer returned a typed error; triage as unsupported boundary or bug. |
| `pdfium_error` | Oracle failed; do not treat as native blocker without another oracle. |
| `both_error` | Usually input policy such as encryption; classify separately from render fidelity. |

## Subsystem Tags

Use the CLI subsystem tags as the stable first-level owner routing:

| Subsystem | Owner area |
| --- | --- |
| `text-fonts` | Font metrics, glyph positioning, fallback rasterization, CMap/ToUnicode fidelity. |
| `annotations-forms` | AcroForm widget appearances, annotation appearance streams, synthesized appearances. |
| `rendering-core` | Operator semantics, table/grid/layout fixture behavior, clipping and default graphics state interactions. |
| `images-color` | Image decode, color conversion, resampling, image-mask and unsupported codec boundaries. |
| `page-geometry` | Page boxes, rotation, user units, scaling, crop and layout transforms. |
| `vector-graphics` | Paths, strokes, line joins/caps, gradients, shadings, patterns, vector stress. |
| `transparency` | Blend modes, alpha constants, transparency groups, soft masks. |
| `optional-content` | OCG/OCMD membership and layer flattening policy. |
| `document-structure` | XRef, incremental update, hybrid references, structural page resolution. |
| `document-security` | Encryption/password policy and security-handler boundaries. |

## Triage Rules

- Keep `accepted_drift` explicit. Do not hide drift by weakening thresholds.
- A blocker row needs a subsystem, fixture family, representative fixtures, and
  a recommended next action.
- A native error is not automatically worse than a visual blocker. Typed
  unsupported errors are valid boundaries when the feature is intentionally out
  of scope for the current release.
- Prefer follow-up slices that can be validated with one focused fixture set
  plus the native-only supported-family gate.
- Do not reintroduce runtime PDFium fallback as a next action. PDFium remains
  maintainer oracle tooling only.
