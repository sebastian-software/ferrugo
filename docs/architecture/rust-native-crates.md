# Rust Native Crate Architecture

Status: accepted initial layout.
Date: 2026-06-24.

The Rust-native renderer grows behind the existing backend-neutral
`pdfrust-thumbnail` facade. PDFium remains the behavior oracle and differential
baseline, but the Rust-native crates must not expose PDFium handles, symbols, or
naming as public API.

## Crates

| Crate | Role | Depends On |
| --- | --- | --- |
| `pdfrust-syntax` | Byte-level PDF syntax, tokens, and offset-aware parser errors. | none |
| `pdfrust-object` | Indirect objects, references, xref data, trailers, catalog, and page tree. | `pdfrust-syntax` |
| `pdfrust-content` | Page content stream tokenization and display-list interpretation. | `pdfrust-object` |
| `pdfrust-render` | Raster buffers, page transforms, drawing, and pixel output helpers. | `pdfrust-content`, `pdfrust-thumbnail` |
| `pdfrust-native` | Rust-native backend adapter for the thumbnail facade. | `pdfrust-object`, `pdfrust-render`, `pdfrust-thumbnail` |

## Dependency Direction

```text
pdfrust-thumbnail
        ^
        |
pdfrust-native -----> pdfrust-render -----> pdfrust-content
        |                                      |
        +---------------> pdfrust-object <-----+
                             |
                             v
                       pdfrust-syntax
```

The direction is intentionally one-way from high-level backend code down to
syntax and object loading. Lower layers do not know about the PDFium backend,
CLI process model, Node packaging, or product distribution.

## Safety Defaults

Rust-native implementation crates start with `#![forbid(unsafe_code)]`.
Performance work can add isolated unsafe modules only after correctness,
profiling, and review justify it. The default implementation style is borrowed
input, typed IDs, checked offsets, bounded decoding, and explicit error values.

## Milestone Boundary

This layout milestone does not parse or render PDFs. It creates stable crate
ownership boundaries so milestones 0022 and later can add behavior in small,
measurable slices against the PDFium baseline.

## Current Syntax Foundation

`pdfrust-syntax` owns borrowed PDF input and offset-aware syntax failures.
`PdfBytes<'a>` and `ByteCursor<'a>` keep scanning over borrowed bytes, while
`ByteOffset`, `SyntaxErrorKind`, and `SyntaxError` provide diagnostics that
later parser layers can preserve without inventing new error plumbing.

The initial primitive parser returns `PdfPrimitive<'a>` values for null,
booleans, numbers, names, literal strings, hexadecimal strings, arrays, and
dictionaries. Names and string contents are borrowed from the original input.
Literal string escapes and hexadecimal string bytes are preserved raw for later
semantic decoding. Parser layers that need to read a primitive followed by more
structure use `parse_primitive_prefix` to keep the first consumed byte offset.

`pdfrust-object` owns typed indirect object IDs and references. Its first loader
can parse contiguous `obj ... endobj` slices into `IndirectObject<'a>` values
and store them in an `ObjectTable<'a>` with duplicate detection.

The classic document loader locates `startxref`, parses classic `xref`
subsections, reads the trailer dictionary, and resolves all in-use xref entries
into the object table. It verifies that each xref offset points at the expected
object ID. Stream objects are represented as `ObjectValue::Stream` with borrowed
raw bytes, dictionary entries, and a bounded decode path for `FlateDecode`,
`ASCIIHexDecode`, and `ASCII85Decode` filter chains.

The modern document loader handles `startxref` values that point at `/XRef`
stream objects. It decodes `/W` and `/Index` entries, loads direct objects from
offset entries, and stores decoded `/ObjStm` buffers separately so compressed
objects can be parsed on demand without self-referential borrows. Hybrid xref
files, indirect stream lengths, and repair mode remain separate milestones.

Both classic and modern document loaders expose `page_tree()`, which resolves
the trailer `/Root`, catalog `/Pages`, page tree `Kids`, inherited page boxes,
and inherited resource references into `PageTree` and `PageMetadata` values.
Content streams and rendering remain separate layers.

`pdfrust-thumbnail` now also owns the backend-neutral `DocumentMetadataBackend`
contract used by the differential harness. `pdfrust-pdfium` implements it by
loading a document through PDFium and reading page count plus page sizes.
`pdfrust-native` implements it through the Rust object model and page tree
without rendering pixels. It also exposes common classic-document inspection
signals for document info fields, XMP presence, outlines, page labels, named
destinations, and tagged-PDF presence. The CLI `compare-metadata` command
records the PDFium oracle and Rust-native candidate results in the baseline
format; `extract-corpus-metadata` serializes the extended native metadata for
corpus work.

`pdfrust-content` starts with a borrowed content-stream tokenizer. It reuses
`pdfrust-syntax` primitives for operands, represents operators as borrowed byte
slices, skips content comments, and preserves byte offsets in `ContentError`.
It deliberately does not execute graphics state, resolve resources, or build a
display list yet.

`pdfrust-render` now owns the first graphics-state and display-list execution
slices. It provides deterministic affine `Matrix` math, a small copyable
`GraphicsState`, stack limits for `q`/`Q`, interpretation for `cm`, `w`, gray
and RGB fill/stroke color, and clipping placeholders. It also builds bounded
path display lists for `m`, `l`, `c`, `h`, `re`, `S`, `s`, `f`, `F`, `f*`,
`B`, and `B*`. Unsupported path construction operators return typed errors.
Page transform creation enforces a default page-raster pixel budget before an
owned RGBA raster buffer can be allocated.
Page `/ExtGState` resources can set path blend mode state for `Normal`,
`Compatible`, `Multiply`, and `Screen`; enabled overprint is rejected as an
explicit unsupported policy until print-production semantics are modeled.
Page `/Shading` resources support the first axial `/ShadingType 2` subset via
`DeviceRGB`/`DeviceGray` Type 2 functions and the `sh` content operator.
Page `/Pattern` resources support the first colored tiling-pattern subset via
indirect `/PatternType 1` streams, `cs /Pattern`, and `/Name scn` fill
selection with bounded repeat sampling.
Text display-list support interprets `BT`, `ET`, `Tf`, `Td`, `Tm`, `Tj`, and
`TJ` into positioned `TextDisplayItem` values. Font descriptors carry simple
single-byte encodings, Differences arrays, and bounded ToUnicode CMaps for the
first character-code mapping layer. Embedded font shaping, glyph outlines, and
searchable text extraction remain later milestones. Rasterization remains a
later milestone.
Image XObject support resolves `/XObject` resources from the object model,
decodes unfiltered, `FlateDecode`, and first-slice `DCTDecode`
`DeviceRGB`/`DeviceGray` image streams within an explicit byte budget, and
stores `ImageDisplayItem` placements using the active CTM. Decoded image
samples are reference-counted so repeated `Do` placements share sample bytes.
`FlateDecode` image streams also apply PNG-style `/DecodeParms /Predictor`
values `10..=15` when the predictor metadata matches the image dimensions and
8-bit component layout. Image `/SMask` references are supported for 8-bit
DeviceGray Image XObjects with dimensions matching the parent image; mask
sample bytes are reference-counted and bounded by a dedicated soft-mask depth
limit. Broader filter chains, CCITT/JPX, unsupported predictors, JBIG2, and
full color management return typed unsupported errors until later corpus-driven
image-filter and color-space milestones.
Form XObject support resolves form streams from `/XObject` resource
dictionaries, decodes form content, applies form matrices, emits bounding-box
clip placeholders, and recursively reuses the path display-list interpreter
with explicit recursion-depth and display-item budgets. Form streams without a
`/Resources` dictionary inherit the caller's XObject scope; form streams with a
local `/Resources /XObject` dictionary use those local names for nested Form
XObjects. Form XObjects with `/Group << /S /Transparency >>` are captured as
path-only transparency-group display items, rasterized into bbox-bounded
transparent intermediate buffers, and composited back into the page under an
explicit pixel budget. Image and text execution inside forms will be wired into
the combined renderer in later rasterization milestones.
The native backend now resolves page-level Form XObjects and paints their path
display-list items into the same page raster. Image and Form resource maps also
track known opposite XObject subtype names so independent image and form passes
do not fail when they encounter each other's `Do` invocations. The form scan
path shares page-level ExtGState resources with the primary path pass so
supported `gs` blend-mode operations remain accepted during repeated content
passes. Mixed ordering and image/text execution inside forms remain later
facade-parity work.
The first raster setup layer defines checked RGBA raster dimensions, an owned
`RasterDevice` with safe row and pixel accessors, and `PageTransform` for
mapping media/crop boxes, rotation, and `max_edge` into device pixels. The
dimension policy intentionally matches the PDFium backend's thumbnail scaling:
scale down only when the rotated page's largest edge exceeds `max_edge`, then
round each target dimension and clamp it to `1..=max_edge`. Actual path, image,
and text rasterization remain later milestones.
Basic path rasterization now paints path display lists into RGBA rasters using
bounded line-segment flattening and fixed supersampling. It supports nonzero
and even-odd fills plus simple stroked line segments, composites opaque device
gray/RGB colors over the requested background with `Normal`, `Multiply`, and
`Screen` blend modes, and is wired through `pdfrust-native` for simple
path-only Classic PDFs. The CLI exposes this path as `render-native` so
generated vector fixtures can be compared against the PDFium backend during
development.
Axial shadings are rasterized by projecting device pixels onto the transformed
gradient axis and sampling the supported Type 2 function directly.
Colored tiling patterns are sampled from decoded path display-list tiles during
fill rasterization and enforce an explicit repeat budget before painting.
Image XObject rasterization draws decoded `DeviceRGB`, `DeviceGray`,
`DeviceCMYK`, and Indexed images with nearest-neighbor sampling through the
image placement matrix and the page transform. The native backend resolves page
`/Resources /XObject` dictionaries, builds image display-list items, and
composites opaque image samples into the same RGBA raster used for paths.
Unsupported filters and color spaces still fail during image resource
resolution with typed errors.
Inline image streams are tokenized as one borrowed `BI`/`ID`/`EI` object so raw
sample bytes do not flow through the generic operator tokenizer. The image
interpreter supports bounded, unfiltered 8-bit `DeviceRGB`/`DeviceGray` inline
images by converting them into the same image display items used for Image
XObjects. Filtered inline images remain explicit unsupported cases until the
image-filter milestone.
Basic text rasterization uses an internal 5x7 ASCII fallback font for the first
visible text milestone. It renders positioned text display-list runs using the
captured text origin, font size, and fill color. `pdfrust-native` resolves page
`/Resources /Font` entries into font descriptors, and the render layer loads
bounded embedded Type1, TrueType, and CFF font program streams behind a small
object-identity cache. Text strings are decoded through simple font encodings
or ToUnicode CMaps and carry their source character codes forward.
Glyph-outline extraction uses `ttf-parser` as the bounded, safe Rust parser for
SFNT-backed TrueType outlines and raw PDF `/FontFile3` CFF streams, converting
`OutlineBuilder` callbacks into the existing path segment representation behind
a small glyph-outline cache with oldest-entry eviction at its configured limit.
Raw CFF streams are passed through
`Face::from_raw_tables` with synthetic required OpenType tables so the native
renderer does not take an extra CFF parser dependency. Type1 `/FontFile`
outlines remain unsupported. The current rasterizer still uses the visible
fallback font; true font-backed rasterization remains a later milestone.
