# Producer Regression Bisect Workflow

Status: accepted for 0190.
Date: 2026-06-29.

Use this workflow when a native renderer gate fails and the maintainer needs to
understand whether the regression is tied to a producer family, feature bucket,
or recent code change.

## Inputs

- A committed manifest with `producer:*` feature tags, such as
  `fixtures/producer-compatibility-manifest.tsv`.
- A native render gate output or failing command.
- Optional local-only corpus aggregates under `fixtures/local-corpus/`.

Do not add private PDFs, private filenames, hashes, screenshots, rendered
pixels, or extracted text to committed artifacts.

## Report Command

Generate the producer-grouped report:

```sh
cargo run -p pdfrust-cli --no-default-features -- producer-regression-report \
  fixtures/generated \
  --manifest fixtures/producer-compatibility-manifest.tsv \
  --max-edge 160 \
  --output target/producer-regression-report.json
```

The report includes:

- summary counts for native renders, fallback-required rows, and errors;
- `producer_groups` from `producer:*` manifest tags;
- `family_groups` from the manifest family column;
- `feature_groups` from manifest feature flags;
- per-record fixture IDs for committed fixtures or redacted local IDs for
  sensitive local-only rows;
- milestone routes for non-native outcomes.

## Bisect Steps

1. Re-run the failing native gate and the producer regression report on the
   same checkout.
2. Identify whether failures cluster by `producer_groups`, `family_groups`, or
   `feature_groups`.
3. If a failure is typed unsupported, route it by `milestone_routes` before
   treating it as a regression.
4. If a previously native producer group now has a fallback or error, bisect the
   smallest renderer or parser range that changed since the last green commit.
5. During `git bisect`, run the smallest command that reproduces the affected
   producer group using `--include-family` when possible.
6. Keep local/private fixture identities out of commits and issue text. Publish
   only aggregate category, producer group, public error class, fallback bucket,
   and synthetic replacement when available.

## Issue Template

Use this shape for a local or GitHub issue:

```text
Producer group:
Manifest:
Fixture id or local id:
Family:
Outcome before:
Outcome after:
Fallback/error bucket:
Affected features:
Milestone route:
Reproduction command:
Privacy check: no private filenames, hashes, text, screenshots, or pixels.
```

## Interpretation

Producer grouping is a triage signal, not a conformance claim for every file
from that application. A producer group becomes actionable when:

- multiple fixtures in that group fail together;
- one producer-specific fixture flips from native to fallback/error;
- the same feature bucket appears across multiple producers;
- a private/local aggregate can be reduced to a committed synthetic fixture.

Unsupported boundaries should stay visible. Do not remove or hide a fixture to
improve headline pass rates.
