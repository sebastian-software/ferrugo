# Native Golden Image Gate

This policy covers the PDFium-free golden image gate run by:

```sh
bash scripts/check_native_golden_images.sh
```

The gate renders a reviewed fixture set with the Rust native backend only, then
compares dimensions, decoded RGBA pixels, and the deterministic PNG artifact
bytes against `fixtures/native-golden-manifest.tsv`. The gate derives the
expected sample count from the manifest instead of hard-coding a release count.

## Manifest Schema

The manifest is TSV with one reviewed baseline per fixture:

| Column | Meaning |
| --- | --- |
| `fixture` | Public fixture path under `fixtures/generated/*.pdf` or `fixtures/real-world/*.pdf`. |
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

- keep the release manifest to reviewed public fixtures that render natively;
- keep `max_edge` at 160 for committed release samples;
- do not commit rendered PNGs, diff images, or private corpus documents;
- store local reports and any temporary rendered artifacts under
  `target/native-golden/`;
- use only public fixtures from `fixtures/generated/` and `fixtures/real-world/`;
- update baselines in the same PR as the renderer change that intentionally
  changes output, with reviewer and date fields refreshed.

Larger visual artifacts belong in local maintainer evidence or separately
retained release notes, not in the repository.

## Current Release Set

The committed manifest covers 33 samples:

- four generated `browser-print` samples;
- four generated `email-web-archive` samples;
- four generated `form` samples;
- four generated `mixed-layout` samples;
- four generated `office-export` samples;
- four generated `presentation` samples;
- four generated `report` samples;
- four generated `scan` samples;
- one public real-world `real-scan` sample.

Other public real-world samples remain in oracle reports when they currently
produce native errors rather than exact native image evidence.
