# PDFium Source Checkout

Status: accepted Phase 0 recipe.
Date: 2026-06-24.

PDFium source is intentionally checked out outside this repository. Keep the
source tree, Chromium dependency cache, and build outputs in a sibling or temp
directory so this repository only stores reproducible instructions and measured
results.

## Pinned Revision

Phase 0 pins PDFium to:

```text
573758fe2dd928279cd52b5a4bc955a6938aab39
```

This revision was read from `https://pdfium.googlesource.com/pdfium.git HEAD`
on 2026-06-24.

## Required Tools

- `git`
- Python 3
- Xcode command line tools on macOS
- Chromium `depot_tools`, providing `gclient`, `gn`, and `ninja`

On macOS arm64, install the Xcode command line tools first:

```sh
xcode-select --install
```

Install `depot_tools` outside this repository:

```sh
mkdir -p ~/src/chromium-tools
git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git \
  ~/src/chromium-tools/depot_tools
export PATH="$HOME/src/chromium-tools/depot_tools:$PATH"
```

## Checkout Directory

Use a sibling directory by default:

```sh
mkdir -p ../pdfium-work
cd ../pdfium-work
```

Expected layout after checkout:

```text
pdfium-work/
  pdfium/
    BUILD.gn
    public/
    testing/
    third_party/
```

## Network Fetch

Run the heavy network steps outside this repository:

```sh
fetch --nohooks pdfium
cd pdfium
git checkout 573758fe2dd928279cd52b5a4bc955a6938aab39
gclient sync --nohooks
gclient runhooks
```

The `fetch` and `gclient sync` steps download Chromium build dependencies. They
are intentionally separate from the later GN/Ninja build commands so failures
can be diagnosed before build configuration starts.

## Revision Verification

From `../pdfium-work/pdfium`:

```sh
git rev-parse HEAD
```

Expected output:

```text
573758fe2dd928279cd52b5a4bc955a6938aab39
```

## External Renderer Adapter

`benchmark-matrix --backend pdfium` and `visual-diff` use PDFium through an
external process. They do not load `libpdfium` through the Rust binding and do
not require the `pdfium` Cargo feature.

Configure the renderer with `--pdfium PATH` or `FERRUGO_PDFIUM_RENDERER`.
When neither is set, the CLI looks for `pdfium_test` on `PATH`.

The command must accept this adapter contract:

```sh
pdfium_test \
  --input fixtures/generated/text-page.pdf \
  --page-index 0 \
  --max-edge 160 \
  --background '#FFFFFFFF' \
  --output target/pdfium-page.ppm
```

The adapter must render one page and write a binary PPM (`P6`) RGB image to the
output path. Keep the adapter script or binary outside this repository unless a
later issue adds a checked-in wrapper.

Missing renderer commands are recorded as `missing-tool` rows in
`benchmark-matrix`. `visual-diff` reports the PDFium side as a reference error
for the affected fixture.

## Local Validation Notes

In this environment on 2026-06-24, `git ls-remote` successfully resolved the
pinned revision. `gclient`, `gn`, and `ninja` were not installed, so the full
checkout and hook run still need to be executed on a machine with
`depot_tools`.
