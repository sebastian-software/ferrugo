#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

out_dir="target/codec-transparency-boundaries"
mkdir -p "${out_dir}"

summary="${out_dir}/codec-transparency-boundaries-summary.txt"

{
  echo "Ferrugo codec and transparency boundary gate"
  echo
} > "${summary}"

echo "==> render-layer deferred image codec tests"
cargo test -p ferrugo-render image_resources_should_report_unsupported_deferred_image_codecs -- --nocapture
echo "render deferred image codec tests passed" >> "${summary}"

echo "==> native image codec deployment tests"
cargo test -p ferrugo-native --no-default-features image_codec_deployment -- --nocapture
echo "native image codec deployment tests passed" >> "${summary}"

echo "==> native image-heavy low-memory tests"
cargo test -p ferrugo-native --no-default-features native_low_memory_profile_should_render_generated_image_heavy_fixtures -- --nocapture
echo "native image-heavy low-memory tests passed" >> "${summary}"

echo "==> render-layer transparency boundary tests"
cargo test -p ferrugo-render ext_graphics_state_resources -- --nocapture
cargo test -p ferrugo-render form_transparency_group_should_enforce_intermediate_pixel_budget -- --nocapture
echo "render transparency boundary tests passed" >> "${summary}"

echo "==> native transparency boundary tests"
cargo test -p ferrugo-native --no-default-features transparency -- --nocapture
cargo test -p ferrugo-native --no-default-features blend_mode -- --nocapture
cargo test -p ferrugo-native --no-default-features extgstate_luminosity -- --nocapture
cargo test -p ferrugo-native --no-default-features transparency_group_band_guard_should_keep_knockout_boundary -- --nocapture
echo "native transparency boundary tests passed" >> "${summary}"

echo "==> supported image codec fallback gate"
cargo run -p ferrugo --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/image-codec-deployment-manifest.tsv \
  --include-family builtin-raster \
  --include-family flate-predictor \
  --include-family mixed-compression \
  --include-family jpeg \
  --include-family mask-alpha \
  --include-family image-heavy \
  --include-family ccitt \
  --fail-on-fallback \
  --max-edge 180 \
  --output "${out_dir}/image-codec-supported.json"
echo "supported image codec fallback gate passed" >> "${summary}"

echo "==> deferred image codec fallback boundary"
cargo run -p ferrugo --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/image-codec-deployment-manifest.tsv \
  --include-family unsupported-specialized \
  --max-edge 180 \
  --output "${out_dir}/image-codec-unsupported.json"

node --input-type=module <<'NODE'
import fs from "node:fs";

const path = "target/codec-transparency-boundaries/image-codec-unsupported.json";
const summary = JSON.parse(fs.readFileSync(path, "utf8"));
const family = summary.families?.["unsupported-specialized"];
if (!family) {
  throw new Error("missing unsupported-specialized family in image codec summary");
}
if (summary.total !== 2 || summary.native_rendered !== 0 || summary.fallback_required !== 2) {
  throw new Error(`unexpected image codec boundary totals in ${path}`);
}
if (summary.fallback_categories?.["image.filter"] !== 2) {
  throw new Error("deferred image codecs must remain typed as image.filter");
}
if (Object.keys(summary.errors ?? {}).length !== 0 || Object.keys(family.errors ?? {}).length !== 0) {
  throw new Error("deferred image codec boundary must not report generic errors");
}
console.log("deferred image codec boundary assertions passed");
NODE
echo "deferred image codec fallback boundary passed" >> "${summary}"

echo "==> supported transparency fallback gate"
cargo run -p ferrugo --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/transparency-conformance-manifest.tsv \
  --include-family alpha \
  --include-family group \
  --include-family blend \
  --include-family image-soft-mask \
  --fail-on-fallback \
  --max-edge 160 \
  --output "${out_dir}/transparency-supported.json"
echo "supported transparency fallback gate passed" >> "${summary}"

echo "==> transparency unsupported fallback boundary"
cargo run -p ferrugo --no-default-features -- summarize-fallbacks fixtures/generated \
  --manifest fixtures/transparency-conformance-manifest.tsv \
  --include-family unsupported-soft-mask \
  --include-family unsupported-blend \
  --max-edge 160 \
  --output "${out_dir}/transparency-unsupported.json"

node --input-type=module <<'NODE'
import fs from "node:fs";

const path = "target/codec-transparency-boundaries/transparency-unsupported.json";
const summary = JSON.parse(fs.readFileSync(path, "utf8"));
for (const name of ["unsupported-soft-mask", "unsupported-blend"]) {
  const family = summary.families?.[name];
  if (!family) {
    throw new Error(`missing ${name} family in transparency summary`);
  }
  if (family.total !== 1 || family.native_rendered !== 0 || family.fallback_required !== 1) {
    throw new Error(`unexpected ${name} totals in ${path}`);
  }
  if (family.fallback_categories?.["graphics.transparency"] !== 1) {
    throw new Error(`${name} must remain typed as graphics.transparency`);
  }
  if (Object.keys(family.errors ?? {}).length !== 0) {
    throw new Error(`${name} must not report generic errors`);
  }
}
if (summary.total !== 2 || summary.native_rendered !== 0 || summary.fallback_required !== 2) {
  throw new Error(`unexpected transparency boundary totals in ${path}`);
}
if (summary.fallback_categories?.["graphics.transparency"] !== 2) {
  throw new Error("transparency unsupported boundaries must remain typed as graphics.transparency");
}
if (Object.keys(summary.errors ?? {}).length !== 0) {
  throw new Error("transparency boundary must not report generic errors");
}
console.log("transparency unsupported boundary assertions passed");
NODE
echo "transparency unsupported fallback boundary passed" >> "${summary}"

echo "==> image codec benchmark gate"
cargo run -p ferrugo --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/image-codec-deployment-manifest.tsv \
  --include-family builtin-raster \
  --include-family flate-predictor \
  --include-family mixed-compression \
  --include-family jpeg \
  --include-family mask-alpha \
  --include-family image-heavy \
  --include-family ccitt \
  --max-edge 180 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output "${out_dir}/image-codec-benchmark.json"
echo "image codec benchmark gate passed" >> "${summary}"

echo "==> transparency low-memory benchmark gate"
cargo run -p ferrugo --no-default-features -- benchmark-native fixtures/generated \
  --manifest fixtures/transparency-stack-memory-manifest.tsv \
  --include-family alpha-stack \
  --include-family group-stack \
  --include-family soft-mask-stack \
  --include-family office-transparency-stack \
  --include-family presentation-transparency-stack \
  --include-family chart-transparency-stack \
  --native-profile low-memory \
  --max-edge 160 \
  --iterations 2 \
  --max-ms 1000 \
  --max-output-bytes 1048576 \
  --output "${out_dir}/transparency-low-memory-benchmark.json"
echo "transparency low-memory benchmark gate passed" >> "${summary}"

echo "==> transparency focused visual comparison"
cargo run -p ferrugo --no-default-features -- visual-diff-poppler fixtures/generated \
  --manifest fixtures/transparency-conformance-manifest.tsv \
  --include-family group \
  --include-family blend \
  --include-family image-soft-mask \
  --max-edge 160 \
  --max-mae 2.0 \
  --max-p95 16 \
  --max-changed-ratio 0.45 \
  --timeout 30 \
  --output "${out_dir}/transparency-poppler-visual-diff.json"
echo "transparency focused visual comparison passed" >> "${summary}"

echo "==> fuzz smoke gate"
bash scripts/check_fuzz_smoke.sh
echo "fuzz smoke gate passed" >> "${summary}"

echo "Codec and transparency boundary gate passed"
echo "Codec and transparency boundary gate passed" >> "${summary}"
