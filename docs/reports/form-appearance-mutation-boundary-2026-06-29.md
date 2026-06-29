# Forms Appearance Mutation Boundary 2026-06-29

Milestone 0194 defines the native renderer boundary between read-only
AcroForm appearance rendering and viewer-side form mutation.

## Scope

- Added `ThumbnailOptions::form_appearance_mode`.
- Default mode `DocumentState` renders source PDF appearance state.
- Explicit `RequestedMutation` mode returns `unsupported` with bucket
  `form.appearance-mutation`.
- Existing widget `/AP` streams and `/AS` appearance-state selections remain
  authoritative over stale `/V` values.
- Synthetic missing-appearance widgets remain bounded thumbnail fallbacks, not
  persisted form updates.

## Fixtures

New generated fixtures:

| Fixture | Purpose | SHA-256 |
| --- | --- | --- |
| `acroform-text-field-stale-appearance.pdf` | `/V` differs from existing `/AP /N`; native renders `/AP`. | `7ffcec4af07060f4f49ce3011146d0757c4ac583052c8d902a21a1af2537d6ee` |
| `acroform-checkbox-stale-appearance-state.pdf` | `/V /Yes` differs from `/AS /Off`; native renders selected `/AS`. | `e08d1cb44cbad87099b8670cc4b2dab2c110642188c2e17055813fbb1a9ed52d` |

Focused manifest:

- `fixtures/form-appearance-mutation-manifest.tsv`

## Native Gate

Command:

```sh
cargo run -p pdfrust-cli -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/form-appearance-mutation-manifest.tsv \
  --include-family existing-appearance \
  --include-family stale-appearance \
  --include-family synthesized-static \
  --fail-on-fallback \
  --output target/form-appearance-0194-supported-gate.json
```

Result:

| Total | Native rendered | Fallback required | Errors |
| --- | --- | --- | --- |
| 9 | 9 | 0 | 0 |

Families:

| Family | Total | Native rendered | Fallback required |
| --- | --- | --- | --- |
| `existing-appearance` | 3 | 3 | 0 |
| `stale-appearance` | 2 | 2 | 0 |
| `synthesized-static` | 4 | 4 | 0 |

## Visual Gate

Document-state visual comparison command:

```sh
cargo run -p pdfrust-cli -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/form-appearance-mutation-manifest.tsv \
  --include-family existing-appearance \
  --include-family stale-appearance \
  --max-mae 8 \
  --max-p95 32 \
  --max-changed-ratio 0.15 \
  --output target/form-appearance-0194-document-state-poppler-diff.json
```

Result:

| Total | Exact | Accepted drift | Blockers | Native errors | Reference errors |
| --- | --- | --- | --- | --- | --- |
| 5 | 0 | 5 | 0 | 0 | 0 |

The full manifest visual run also rendered all native rows, but the
`synthesized-static` family is not a Poppler parity gate: Poppler renders some
missing-appearance widgets blank and timed out on one checkbox fallback. That
matches the documented policy that native synthesis is a bounded thumbnail
fallback, not persisted form editing.

## API Behavior

Targeted native tests verify:

- Stale text-field `/V` does not override an existing `/AP /N`.
- Stale checkbox `/V` does not override selected `/AS /Off`.
- `FormAppearanceMode::RequestedMutation` returns bucket
  `form.appearance-mutation`.
- Input bytes remain unchanged after a rejected mutation request.

Commands:

```sh
cargo test -p pdfrust-native acroform -- --nocapture
cargo test -p pdfrust-native appearance -- --nocapture
cargo test -p pdfrust-native mutation -- --nocapture
```

Results:

- `acroform`: 9 passed.
- `appearance`: 19 passed.
- `mutation`: 1 passed.
