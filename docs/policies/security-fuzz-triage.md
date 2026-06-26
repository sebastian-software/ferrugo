# Security Fuzz Triage Policy

Status: accepted for 0178.
Date: 2026-06-26.

Fuzzing is a repeatable maintenance loop for untrusted PDF input. The goal is
not to make every malformed file render; the goal is to prevent panics, aborts,
unbounded allocation, uncontrolled CPU work, and unstable error boundaries.

## Target Matrix

| Target | Boundary | Security focus |
| --- | --- | --- |
| `primitive_parse` | Syntax primitive parsing and prefix parsing. | Nesting, malformed scalars, offset accounting. |
| `xref_load` | Indirect object parsing, classic xref, xref streams, object streams. | Object graph corruption, offset drift, expansion limits. |
| `stream_decode` | Stream object parsing and bounded filter decoding. | Decode expansion and malformed filter data. |
| `content_tokenize` | Content stream tokenization and inline images. | Unterminated data, operand/operator ambiguity. |
| `render_setup` | Native metadata inspection and first-page render setup. | Page setup, declared image dimensions, renderer budgets. |

`scripts/check_fuzz_smoke.sh` runs the current matrix and is the local/nightly
smoke entry point.

## Finding Classes

| Class | Examples | Required outcome |
| --- | --- | --- |
| Security risk | Panic, abort, uncontrolled allocation, non-terminating work. | Fix or add a hard budget before accepting the input path. |
| Correctness risk | Stable input returns inconsistent classes or corrupts later work. | Add a minimized regression and classify as supported, malformed, or unsupported. |
| Unsupported input | Valid PDF feature outside current native scope. | Return a typed unsupported bucket; document if user-visible. |
| Malformed input | Broken PDF structure or invalid stream syntax. | Return a typed malformed error unless bounded recovery is explicitly accepted. |

## Crash Artifact Workflow

1. Keep private crash artifacts outside the repository until review.
2. Minimize the artifact with the relevant target and local reproduction command.
3. Remove private document payload whenever possible; prefer synthetic reduced
   inputs under `fixtures/adversarial/`.
4. Add or update a regression test before or with the fix.
5. Record the final classification in the relevant report or policy.

Crash artifacts must not be published when they contain customer data,
credentials, signatures, private text, or proprietary embedded assets.

## Regression Rules

Resolved fuzz findings should leave one of:

- a unit test for the exact parser, object, content, image, font, or raster
  budget boundary;
- a reduced adversarial fixture under `fixtures/adversarial/`;
- a documented unsupported bucket when the input is valid but outside scope;
- a documented malformed boundary when repair is intentionally rejected.

For resource exhaustion findings, the regression must prove that the failure
happens before the large allocation or unbounded loop.

## Nightly Gate

The nightly or local smoke loop should run:

```sh
bash scripts/check_fuzz_smoke.sh
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Long-running fuzz campaigns can use the same targets, but their crash corpus
must go through the artifact workflow before anything is committed.
