# Fixture Policy

Status: accepted Phase 0 policy, updated for #97 real-world seed tier.
Date: 2026-07-04.

Fixtures committed to this repository must be license-safe, small, and easy to
inspect. Generated fixtures remain the default because they are reproducible
without network access. A small real-world tier is allowed only when the source
is public and redistributable, provenance is documented per file, and the
fixture fills a benchmark coverage gap that generated reductions cannot.

## Committed Fixtures

Generated committed fixtures live under `fixtures/generated/` and must meet
these rules:

- Generated from repository scripts or short handwritten source.
- Small enough for review; prefer simple one-page files.
- Focused on one behavior per fixture.
- Regenerable without network access.

Real-world committed fixtures live under `fixtures/real-world/` and must meet
these rules:

- Public source URL recorded in `fixtures/real-world/README.md`.
- Redistribution license recorded per file and per manifest row.
- No private, customer, user-supplied, or legally ambiguous PDFs.
- Small enough for normal repository checkout and benchmark review.
- SHA-256, page count, and producer notes recorded before commit.

`fixtures/corpus-manifest.tsv` assigns each committed fixture to a corpus
family and records source, license, page-count, feature, and note metadata.
See `docs/corpus-taxonomy.md` for family definitions and private sampling
rules.

Reduced adversarial inputs live under `fixtures/adversarial/`. They are used by
fuzz-smoke targets and regression tests for panic prevention, bounded parsing,
and stable malformed-error mapping. These files should stay small and
minimized; see `docs/fuzzing.md`.

The initial seed set covers:

- page size
- text drawing
- vector path drawing
- vector-heavy chart stress drawing with nested clips and cubic curves
- image placement through an inline image stream
- image placement through an Image XObject
- CMYK and Indexed image color spaces
- DeviceRGB rendering with catalog OutputIntent metadata
- DCT/JPEG Image XObject decoding
- Flate image PNG predictor decoding
- image soft-mask alpha compositing
- page-sized scan-like DeviceGray image placement
- mixed text, image, and vector marker page
- page-targeted rendering with an unrelated malformed page stream
- Form XObject invocation
- path-only Form XObject transparency groups
- ExtGState Multiply and Screen path blend modes
- ExtGState fill and stroke alpha constants
- dashed vector strokes
- stroke line-cap styles
- stroke line-join styles
- even-odd path clipping
- normal annotation appearance streams
- annotations without usable appearance streams
- link annotation appearance streams
- text-markup highlight annotation appearance streams
- widget annotation appearance streams
- AcroForm text-field widget appearance streams
- AcroForm stale widget appearances where `/V` differs from existing `/AP`
- AcroForm checkbox widget appearance state dictionaries
- AcroForm stale checkbox states where `/V` differs from selected `/AS`
- AcroForm radio widget appearance state dictionaries
- AcroForm signature placeholder widget appearance streams
- Type0 CID font with Identity-H and ToUnicode mapping
- vertical CJK Type0 CID font with Identity-V and ToUnicode mapping
- pre-positioned RTL Type0 CID text with ToUnicode mapping
- optional content group default-visible and default-hidden layers
- unsupported optional content membership dictionaries
- classic incremental update object revisions
- hybrid-reference classic xref plus xref stream entries
- encrypted trailer placeholder documents
- malformed xref object-offset drift
- axial DeviceRGB shading gradients
- radial DeviceRGB shading gradients
- colored tiling patterns
- embedded TrueType font resource resolution
- ToUnicode text character-code mapping
- Encoding Differences array mapping
- text spacing, `TJ` fragmentation, and invisible text rendering mode
- office-style ruled table layout with header fill and text cells
- two-page report layout with repeated headers, table lines, and text

## Local Corpora

Private, reference-only, or license-unclear real-world PDFs must stay out of
Git. Store them under `fixtures/local-corpus/` and describe them with
`fixtures/local-corpus.example.toml` before running local measurements. The
metadata is aggregate-only and is validated with:

```sh
cargo run -p ferrugo --no-default-features -- validate-local-corpus \
  fixtures/local-corpus/metadata.toml --allow-missing
```

Do not commit:

- PDFs from users or private documents
- proprietary sample packs
- large public corpora
- public PDFs without an explicit redistribution basis
- generated PNG outputs from local measurements unless a milestone explicitly
  asks for a small committed artifact

Local PDFs can feed the same benchmark-matrix schema by creating an untracked
TSV shaped like `fixtures/local-corpus-matrix.example.tsv` and passing it with
`--manifest`. Keep concrete local filenames, hashes, text, screenshots, and
rendered pixels out of published reports unless they have completed a separate
redistribution review.

## Regeneration

Run:

```sh
python3 scripts/generate_fixtures.py
```

The generator writes deterministic PDFs into `fixtures/generated/`.
