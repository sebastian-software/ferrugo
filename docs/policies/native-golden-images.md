# Native Golden Image Gate

This policy covers the PDFium-free golden image gate run by:

```sh
bash scripts/check_native_golden_images.sh
```

The gate renders a tiny reviewed fixture set with the Rust native backend only,
then compares dimensions, decoded RGBA pixels, and the deterministic PNG artifact
bytes against `fixtures/native-golden-manifest.tsv`.

## Manifest Schema

The manifest is TSV with one reviewed baseline per fixture:

| Column | Meaning |
| --- | --- |
| `fixture` | Public generated fixture path under `fixtures/generated/*.pdf`. |
| `family` | Corpus family represented by the fixture. |
| `backend` | Must be `rust-native`. |
| `platform_os` | Target platform or `any` for platform-neutral generated fixtures. |
| `platform_arch` | Target architecture or `any` for platform-neutral generated fixtures. |
| `page_index` | Rendered page index. |
| `max_edge` | Native thumbnail maximum edge. |
| `width`, `height` | Expected rendered output dimensions. |
| `decoded_pixel_hash` | FNV-1a-64 fingerprint of decoded RGBA bytes. |
| `encoded_artifact_hash` | FNV-1a-64 fingerprint of the deterministic PNG bytes. |
| `hash_algorithm` | Must be `fnv1a64`. This is a regression fingerprint, not a security hash. |
| `tolerance_policy` | Must be `exact` for the release gate. |
| `reviewer`, `reviewed_at` | Review ownership for the committed baseline. |
| `notes` | Short fixture intent. |

`compare-golden` writes JSON with `report_kind:
native-golden-comparison`. Each failing record includes a concrete `reason`,
for example a dimension mismatch, decoded pixel hash drift, encoded artifact
hash drift, render error, missing fixture, or manifest/platform mismatch.

## Retention Policy

Committed golden evidence stays hash-only:

- keep the release manifest to five samples unless a release blocker requires a
  targeted addition;
- keep `max_edge` at 160 for committed release samples;
- do not commit rendered PNGs, diff images, or private corpus documents;
- store local reports and any temporary rendered artifacts under
  `target/native-golden/`;
- use only public generated fixtures from `fixtures/generated/`;
- update baselines in the same PR as the renderer change that intentionally
  changes output, with reviewer and date fields refreshed.

Larger visual artifacts belong in local maintainer evidence or separately
retained release notes, not in the repository.

## Initial Release Set

The first committed manifest covers:

- `browser-print`: `browser-print-clipped-backgrounds.pdf`;
- `office-export`: `office-table.pdf`;
- static `form`: `flattened-form-export.pdf`;
- `scan`: `scanner-skewed-mailroom-page.pdf`;
- PDF 2.0 accepted basics: `pdf20-basic-office.pdf`.
