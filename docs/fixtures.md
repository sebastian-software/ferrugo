# Fixture Policy

Status: accepted Phase 0 policy.
Date: 2026-06-24.

Fixtures committed to this repository must be generated, license-safe, small,
and easy to inspect. They exist to exercise thumbnail plumbing, not to claim
full PDF feature coverage.

## Committed Fixtures

Committed fixtures live under `fixtures/generated/` and must meet these rules:

- Generated from repository scripts or short handwritten source.
- No private, customer, user-supplied, scanned, or licensed third-party PDFs.
- Small enough for review; prefer simple one-page files.
- Focused on one behavior per fixture.
- Regenerable without network access.

The initial seed set covers:

- page size
- text drawing
- vector path drawing
- image placement through an inline image stream

## Local Corpora

Real-world PDFs are useful for manual probes, but they must stay out of Git.
Store them under `fixtures/local-corpus/` and describe them with
`fixtures/local-corpus.example.toml` before running local measurements.

Do not commit:

- PDFs from users or private documents
- proprietary sample packs
- large public corpora
- generated PNG outputs from local measurements unless a milestone explicitly
  asks for a small committed artifact

## Regeneration

Run:

```sh
python3 scripts/generate_fixtures.py
```

The generator writes deterministic PDFs into `fixtures/generated/`.
