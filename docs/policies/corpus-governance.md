# Corpus Governance Policy

Status: accepted for 0179.
Date: 2026-06-26.

Corpus growth must preserve provenance, licensing clarity, privacy boundaries,
and regression visibility. Coverage numbers are useful only when unsupported
features, failures, performance budgets, and fixture ownership remain visible.

## Fixture Metadata

Every committed manifest entry must include:

- path;
- family;
- source;
- license or permission status;
- page count;
- feature tags;
- notes with expected behavior or ownership context.

Generated fixtures should name the generator script. Public fixtures must record
the public source and license. Private or local-only PDFs must not be committed;
they should use aggregate metadata through the local corpus format.

## Review Rules

Adding a fixture requires:

- provenance and license review;
- expected native outcome or unsupported bucket;
- family assignment that matches the intended coverage signal;
- a clear milestone, issue, or report reference when the fixture represents a
  regression.

Removing a fixture requires:

- confirming another fixture still covers the same feature or bug;
- preserving historical report context when removal changes coverage numbers;
- documenting the reason in the relevant milestone or report.

## Regression Visibility

Regression entries should remain visible with:

- owner or milestone reference;
- category: correctness, unsupported feature, malformed input, performance,
  memory, or privacy;
- severity: blocker, release-risk, follow-up, or accepted boundary;
- current status: open, mitigated, fixed, or documented unsupported.

Do not hide unsupported cases to improve headline native coverage. Unsupported
buckets are release-decision data.

## Dashboard Flow

`scripts/generate_corpus_dashboard.sh` is the native-only maintainer entry
point. It writes only to `target/corpus-dashboard/` by default and produces:

- `metadata.json`;
- `local-corpus-validation.json`;
- `support.json`;
- `operators.json`;
- `performance.json`;
- `batch.json`;
- `dashboard.json`.

The dashboard is not a committed source of truth. It is a generated decision
artifact for release and milestone reviews.

## Producer Regression Flow

`ferrugo-cli producer-regression-report` is the native-only local report for
producer-scoped regression triage. It requires a manifest and groups render
outcomes by `producer:*` feature tags, manifest family, and feature flags.

The report is allowed to include committed generated fixture paths so maintainers
can reproduce failures locally. Private or local-only fixtures must remain
redacted to local fixture IDs and must not publish original filenames, hashes,
text, screenshots, or rendered pixels.

Use producer regression reports when a gate changes native outcome status and
the maintainer needs to identify whether the break is producer-specific,
feature-specific, or a broader renderer regression.

## Privacy Boundary

Private local corpus reporting must stay aggregate-only. Do not publish private
filenames, hashes, extracted text, screenshots, rendered pixels, signatures, or
customer-specific labels. Use synthetic reduced fixtures when a private finding
needs to become a public regression.
