# Greenfield Architecture Review (External Research)

Date filed: 2026-07-03
Origin: independent external research on the optimal design of a greenfield
Rust PDF→PNG renderer, produced without knowledge of ferrugo's actual state.
Filed verbatim below the reconciliation section; the original text is German.

## Reconciliation with ferrugo as of 2026-07-03 (main `ec6b335`)

The document's core thesis — typed display-list IR, band/tile rendering,
coverage rasterizer, aggressive caches, CPU-first with optional GPU later —
matches what ferrugo independently built in issues #4–#9 and #23–#31:

| Doc priority | Status in ferrugo |
| --- | --- |
| 1. Tile/band rendering | Done: default serial banding ≥ 160,000 px (#28), opt-in 2/4/8-worker parallel bands (#29), pixel-bounds culling, transparency-group guards |
| 2. Typed display-list IR | Done since inception; the IR sketch in the doc closely matches the real display list |
| 3. ARM64/NEON SIMD kernels | **Open — tracked in #50** (the per-draw span blitters from #7 are the designed insertion point) |
| 4. Image-only / scanned fast path | **Open — tracked in #51** (no page classifier or direct-image route exists yet) |
| 5. Glyph cache + font pipeline | Done (#8/#27): glyph bitmap, font-resource, ICC, Type3 template + rendered caches with budgets; glyph-outline reuse deferred until an outline-backed text path exists (the doc's skrifa/swash pointers apply then) |
| 6. Streaming decode/encode | Decode side partial (session caches, lazy resolution; no mmap/arenas). Encode side **open — tracked in #52** (output buffer is still fully assembled; bands→rows→PNG is the natural completion) |
| 7. Optional hybrid GPU | Deliberately deferred, matching the doc's own recommendation; Vello Hybrid noted as the candidate when data justifies it |
| 8. Coverage/sparse rasterizer | Done in the recommended lineage (signed-area scanline cells, #5/#25); sparse strips remain a possible later step |

Where the document is behind reality: its "realistic edge" projection
(1.2–2× on ARM64 office workloads, 2–5× lower peak RAM, global wins
unrealistic) is already exceeded on the measured corpus — ferrugo beats
Poppler 1.7–12× cold at 3–7× lower RSS, and the refreshed two-run PDFium
matrix (#30, `docs/reports/pdfium-comparison-refresh-2026-07-03.md`) shows
ferrugo below PDFium in hot-render p95 and peak RSS in every measured
family. Caveats stay: synthetic corpus, small raster targets, one host.

Actionable takeaways were filed as issues #50 (SIMD row-blitter kernels),
#51 (page classifier + direct-image fast path), #52 (streaming PNG from
raster bands). The doc's benchmark-category list and the positioning claim
(ARM64-first, workload-aware fast paths) remain good inputs for docs and
README positioning.

---

Verstanden: **greenfield PDF→PNG Renderer in Rust**, nicht PDFium/MuPDF/Poppler einbinden. Meine Einschätzung nach Recherche: Eine Performance-Edge entsteht nicht primär durch „Rust“ oder „Zero Copy“, sondern durch eine **andere Rendering-Architektur mit aggressiven Fast Paths**. PDF ist groß und fies: Transparenz, Soft Masks, Blend Modes, isolierte Gruppen, Bildmasken, ICC-Farbräume, Fonts, Type3, Pattern/Shadings usw. sind echte Komplexitätstreiber; Transparenz wurde mit PDF 1.4 eingeführt und umfasst u. a. Alpha-Compositing, Soft Masks, Blend Modes und Transparenzgruppen. ([blog.adobe.com][1])

## Kernthese

Der aussichtsreichste Weg wäre:

**PDF Parser → typed Display-List/IR → Workload-Klassifikation → Tile/Band Scheduler → SIMD-CPU Rasterizer + Spezialpfade für Bilder/Text + optional Hybrid-GPU → Streaming PNG Encoder**

Also nicht „PDF interpretieren und direkt malen“, sondern erst eine **kompakte, optimierbare Zwischenrepräsentation** erzeugen. MuPDF und PDF.js machen konzeptionell ebenfalls Display-List/Operator-List-artige Zwischenschritte; MuPDF rendert beispielsweise Display Lists in Pixmaps, PDF.js hat eine `PDFOperatorList` mit `fnArray` und `argsArray`. ([mupdf.readthedocs.io][2]) ([mozilla.github.io][3])

Der Unterschied im Greenfield-Ansatz wäre: Diese IR von Anfang an **cache-freundlich, tilebar, SIMD-fähig und workload-spezialisiert** bauen.

---

# 1. Tile-/Band-basiertes Rendering ist vermutlich der größte Hebel

PDF-Seiten können riesig sein, und eine naive RGBA-Fläche für 300 DPI oder mehr frisst schnell Speicherbandbreite. Der moderne Ansatz wäre:

1. Page Content in Display Commands kompilieren.
2. Für jedes Kommando Bounding Boxes berechnen.
3. Commands in Tiles/Bands einordnen.
4. Tiles parallel rendern.
5. Nur sichtbare/berührte Tiles bearbeiten.
6. Transparenzgruppen separat behandeln.

Das gibt dir mehrere Vorteile:

* **Single-page parallelism**: Nicht nur mehrere Seiten parallel rendern, sondern eine einzelne komplexe Seite.
* **Bessere Cache Locality**: Kleine Tile Buffers passen besser in CPU-Caches.
* **Weniger Speicher-Peak**: Bandweise PNG-Ausgabe möglich.
* **Culling**: Objekte außerhalb des Zielbereichs werden gar nicht erst gerastert.
* **Skalierbarkeit auf ARM-Server**: viele Kerne, aber Speicherbandbreite bleibt teuer.

MuPDF kann bereits mehrere Seiten parallelisieren, aber die Dokument-/Page-Load-Phase ist begrenzt; Display Lists können dann in anderen Threads genutzt werden. Das zeigt genau, wo ein Greenfield-Renderer ansetzen müsste: nicht nur Page-Level Parallelism, sondern noch aggressiver innerhalb der Seite. ([mupdf.readthedocs.io][2]) ([mupdf.readthedocs.io][2])

**Architektonisch wichtig:** Transparenzgruppen und Blend Modes machen Tiling schwierig, weil PDF-Malreihenfolge semantisch relevant ist. Der Trick wäre daher nicht „alles beliebig parallel“, sondern:

* normale opaque Objekte stark parallelisieren,
* transparente Gruppen als eigene temporäre Layer rendern,
* Gruppen-Bounds minimal halten,
* `Normal + opaque` als Ultra-Fast-Path,
* exotische Blend Modes in langsameren Spezialpfad verschieben.

Das lohnt sich sehr wahrscheinlich mehr als reine Parser-Optimierung.

---

# 2. Eine eigene typed IR statt generischer PDF-Interpreter-Schleife

Eine performante Greenfield-Engine sollte PDF-Operatoren nicht dauerhaft als dynamische Interpreter-Operationen ausführen. Besser wäre ein Compile-Schritt:

```text
PDF content stream
  -> interpreted graphics state
  -> normalized typed IR
  -> tile bins
  -> raster jobs
```

Beispielhafte IR-Kommandos:

```text
DrawImage {
  object_id,
  decoded_format,
  src_rect,
  dst_quad,
  opacity,
  mask,
  colorspace,
  interpolation,
  bbox
}

FillPath {
  path_id,
  fill_rule,
  transform,
  paint,
  blend_mode,
  opacity,
  clip_id,
  bbox
}

DrawGlyphRun {
  font_id,
  glyphs,
  positions,
  transform,
  fill,
  render_mode,
  bbox
}

BeginTransparencyGroup { ... }
EndTransparencyGroup { ... }
```

Der Vorteil: Du kannst schon vor dem Rasterizing entscheiden:

* Ist die Seite im Wesentlichen ein einzelnes Vollseitenbild?
* Gibt es nur schwarzen Text auf weißem Hintergrund?
* Gibt es Transparenz?
* Gibt es weiche Masken?
* Gibt es komplexe Vektorgeometrie?
* Gibt es wiederverwendbare XObjects?
* Lohnt GPU?
* Reicht CPU-SIMD?

Hayro ist als Rust-Referenz interessant, weil es PDF in abstrakte Devices interpretiert und daraus z. B. Bitmaps erzeugt; zugleich dokumentiert Hayro selbst, dass Performance bisher nicht der Fokus ist und dass PDF-Correctness viele harte Edge Cases enthält. Genau da könnte ein neues Projekt ansetzen: gleiche Problemklasse, aber IR und Backend von Anfang an auf Performance trimmen. ([Docs.rs][4]) ([Docs.rs][5])

---

# 3. ARM64/NEON ist ein sehr guter Spezialisierungswinkel

ARM-Optimierung ist tatsächlich aussichtsreich, besonders für:

* Alpha Blending
* Premultiplied-Alpha-Konvertierung
* Image Scaling
* Bilinear/Bicubic Sampling
* Mask Composition
* PNG Filter
* Farbkonvertierung RGB/CMYK/Gray
* Gamma/Transfer-Funktionen
* Path Coverage Compositing
* Glyph Mask Blitting

Rusts `std::simd` ist zwar spannend, aber weiterhin als experimentell/nightly dokumentiert. Für stabile Rust-Implementierungen sind aktuell eher `std::arch`, eigene NEON-Kernel oder Abstraktionen wie `rten-simd` interessant; `rten-simd` unterstützt u. a. AVX2/AVX-512, Arm Neon und WebAssembly SIMD mit Runtime Dispatch. ([Rust-Dokumentation][6]) ([Docs.rs][7])

Meine klare Empfehlung: **ARM64 nicht als Nebenprodukt behandeln**, sondern als First-Class Target.

Das bedeutet konkret:

```text
backend_scalar
backend_aarch64_neon
backend_x86_64_avx2
backend_x86_64_avx512 optional
```

Für Apple Silicon und AWS Graviton/Neoverse solltest du separat benchmarken. Beide sind ARM64, aber Cache, Speicherbandbreite und SIMD-Durchsatz unterscheiden sich stark.

Der größte Performance-Gewinn kommt vermutlich bei Compositing-Kernels:

```text
dst = src + dst * (1 - src_alpha)
```

Das ist millionen- bis milliardenfach pro Dokumentbatch relevant. Wenn dein Renderer dort besser ist als generische Lösungen, merkt man das massiv.

---

# 4. Bildlastige PDFs brauchen eigene Fast Paths

Viele PDF→PNG-Workloads sind gar nicht primär Vektor-Rendering, sondern:

* gescannte PDFs,
* Rechnungen,
* OCR-PDFs,
* Dokumente mit großem JPEG/JPEG2000/JBIG2-Bild plus unsichtbarem Text,
* Präsentations-Exports mit großen Rasterbildern.

Für solche Seiten wäre ein „normaler Renderer“ unnötig teuer. Ein Greenfield-Renderer sollte erkennen:

```text
Page = white background + one full-page image + optional invisible OCR text
```

Dann kann man:

* das Bild direkt decodieren,
* auf Zielauflösung skalieren,
* Text ignorieren, wenn unsichtbar,
* Compositing vermeiden,
* eventuell direkt zeilenweise in PNG schreiben.

PDF-Bilder können über diverse Filter kommen, z. B. DCT/JPEG, JPX/JPEG2000, JBIG2, CCITT Fax, Flate mit Predictor usw.; `rpdfium_codec` dokumentiert genau diese PDF-Stream-Filter und bietet sogar scanline-basiertes Decoding. ([Docs.rs][8])

Das ist einer der realistischsten Wege zu einer **messbaren Edge**: Nicht PDF allgemeiner rendern, sondern häufige PDF-Typen sehr viel schneller erkennen und abkürzen.

Bei gescannten Dokumenten könnte das Ziel sein:

```text
PDF page image stream -> decoder -> scaler -> PNG rows
```

statt:

```text
PDF page -> RGBA canvas -> composite image -> full framebuffer -> PNG
```

Das ist kein „Zero Copy“ im absoluten Sinn, weil komprimierte Streams decodiert werden müssen. Aber es ist **zero unnecessary copy**: keine unnötigen Zwischenflächen, keine unnötige RGBA-Konvertierung, kein Full-Page-Framebuffer, wenn zeilenweises Arbeiten reicht.

---

# 5. Zero Copy ist hilfreich, aber nicht der Hauptgewinn

Zero Copy lohnt sich vor allem in diesen Bereichen:

* `mmap`/`bytes` für PDF-Datei und Objektstreams,
* lazy object resolution,
* borrowed slices für unkomprimierte Streamdaten,
* arenas für Page-IR,
* interner Resource Cache statt wiederholter Decodes,
* row/tile streaming in den PNG-Encoder.

Aber PDF-Streams sind oft komprimiert. Fonts, Bilder, Content Streams und Object Streams müssen häufig decodiert werden. „Zero Copy“ heißt daher praktisch:

> Nicht versuchen, alles ohne Kopie zu machen, sondern jede Kopie erzwingen lassen durch Semantik, nicht durch Architektur.

Guter Greenfield-Ansatz:

```text
ByteSource: mmap / Arc<[u8]>
ObjectResolver: lazy, cache-aware
StreamDecoder: pull-based / scanline when possible
PageIR: arena-allocated, compact
RasterJob: tile-local
Output: streaming rows
```

Rust-Crates wie `pdf_syntax` zeigen, dass ein leichtgewichtiger Low-Level-PDF-Zugriff mit XRefs, Object Streams, Stream-Decoding und Page-Iteration möglich ist; explizit höhere Themen wie Fonts/Farbräume liegen dort außerhalb des Scopes. ([Docs.rs][9])

Für greenfield wäre `pdf_syntax` entweder Referenz oder unterste Schicht — aber nicht die eigentliche Performance-Story.

---

# 6. Glyph/Text-Rendering: Cache aggressiv, aber PDF-Fonts sind hart

Text ist performance-relevant, aber PDF-Text ist semantisch unangenehm:

* eingebettete Fonts,
* subset Fonts,
* CID/CMaps,
* Type3 Fonts,
* Glyph-Positionierung über PDF-Operatoren,
* Text Rendering Modes,
* Stroke/Fill/Clip,
* fehlende oder kaputte ToUnicode Maps.

Für Performance brauchst du:

```text
Font object id + glyph id + size/matrix + render mode -> cached glyph mask/path
```

Bei vielen Office-PDFs wiederholen sich Fonts und Glyphen extrem. Ein guter Glyph Cache spart sehr viel.

Rust-seitig sind Fontations/Skrifa und Swash spannend. Skrifa ist ein Rust/OpenType-Crate für Font-Metadaten und Glyph-Outlines; Chrome arbeitet mit Fontations/Skrifa als Rust-Ersatzpfad für FreeType/Font-Handling in Skia. ([Docs.rs][10]) ([Chrome for Developers][11]) Swash bietet Font-Introspection, Shaping und Glyph-Rendering in Rust mit Fokus auf wenig transienten Heap-Allocations. ([GitHub][12])

Aber: PDF-Text ist nicht normales Browser-Textlayout. Für PDF brauchst du weniger „Layout Engine“ und mehr:

* PDF-Encoding/CMap-Auflösung,
* Glyph-ID-Zugriff,
* exakte Glyph-Positionierung,
* Masken-/Outline-Rasterung,
* Caching.

---

# 7. GPU lohnt sich — aber nicht pauschal

Ein optionaler GPU-/Hybrid-Pfad kann sehr sinnvoll sein, besonders für:

* vektorlastige Seiten,
* Karten/CAD,
* viele Pfade,
* große Transparenzflächen,
* große Gradients/Shadings,
* sehr hohe DPI.

Vello ist hier als Rust-Technologie interessant: Vello ist ein GPU-compute-zentrierter 2D-Renderer auf Basis von `wgpu`; `wgpu` abstrahiert u. a. Vulkan, Metal, D3D12, OpenGL/WebGPU. ([GitHub][13]) ([Docs.rs][14])

Noch spannender ist der neuere Hybrid-Gedanke: Vello Hybrid kombiniert CPU-Pfadverarbeitung mit GPU-Compositing/Blending und versucht, Datentransfer zu minimieren. ([Docs.rs][15])

Aber für PDF→PNG gibt es einen wichtigen Haken:

> Am Ende musst du das Bild wieder zur CPU zurückholen und PNG encoden.

GPU lohnt sich also nicht automatisch. Bei kleinen Seiten oder bildlastigen PDFs kann GPU langsamer sein als ein guter CPU-SIMD-Pfad. Ich würde GPU daher nicht als Default bauen, sondern als auswählbaren Backend-Pfad nach Klassifikation:

```text
if page.vector_complexity_high && output_dpi_high && gpu_available:
    use hybrid_gpu
else:
    use cpu_simd_tiles
```

Die beste Architektur wäre backend-neutral:

```text
IR -> CPU Raster Backend
IR -> Hybrid GPU Backend
IR -> Direct Image Backend
```

---

# 8. Vektor-Rasterizer: sparse strips / tile coverage statt klassischer Scanline-only

Für echte Performance-Edge bei Vektorgeometrie würde ich mir die moderne Vello-CPU/Vello-Sparse-Strips-Richtung sehr genau ansehen. Vello CPU ist ein Rust-2D-Renderer; die Linebender-Benchmarks zeigen auf Apple M1 Pro teils sehr starke Ergebnisse gegenüber Skia/Cairo, besonders bei größerer Geometrie, und die Arbeiten fokussieren explizit SIMD und Multithreading. ([Linebender][16])

Das Konzept „Sparse Strips“ ist für PDF interessant, weil PDFs oft große weiße/solide Flächen und lokal komplexe Bereiche haben. Sparse/Tiled Coverage vermeidet Arbeit in Regionen, die ohnehin leer oder konstant sind. Die Vello-Dokumentation beschreibt gemeinsame Tiling-/Geometry-Komponenten für CPU/Hybrid-Renderer. ([GitHub][17]) ([Docs.rs][18])

Für greenfield heißt das:

* nicht einfach Anti-Grain/Skia-artigen Rasterizer nachbauen,
* sondern coverage-orientiert, tile-orientiert, sparse denken,
* Pfade in eine Form bringen, die SIMD und Parallelität gut verträgt.

---

# 9. PNG-Encoding darf nicht vergessen werden

Bei schnellen Renderern kann PNG-Encoding plötzlich dominieren. Daher:

* PNG zeilenweise schreiben,
* Filterstrategie benchmarken,
* Kompression konfigurierbar machen,
* bei Batch-Konvertierung Rendering und Encoding pipelinen,
* optional „fast PNG“ statt maximaler Kompression,
* Framebuffer-Format so wählen, dass keine teure Kanalumordnung nötig ist.

Für PDF→PNG ist „schneller Renderer“ nur die halbe Wahrheit. Die User messen meist:

```text
PDF bytes -> PNG file written
```

nicht nur Rasterizer-Zeit.

Rust-seitig sind `zune-*`-Codecs interessant; `zune-png` dokumentiert z. B. schnelle Inflate-Pfade und platform intrinsics. ([Docs.rs][19])

---

# Priorisierte Mechaniken

| Priorität | Mechanik                               | Warum es sich lohnt                                        | Risiko                                   |
| --------: | -------------------------------------- | ---------------------------------------------------------- | ---------------------------------------- |
|         1 | **Tile/Band Rendering**                | Parallelität, Cache Locality, weniger Peak-RAM, Culling    | Transparenzgruppen korrekt handhaben     |
|         2 | **Typed Display-List/IR**              | Optimierbar, klassifizierbar, backend-neutral              | PDF-Grafikzustand korrekt normalisieren  |
|         3 | **ARM64 NEON Kernels**                 | Blending, Masking, Scaling, Color Conversion sind Hotspots | Viel Benchmark-/Kernelarbeit             |
|         4 | **Image-only / scanned-PDF Fast Path** | Sehr häufiger Real-World-Case; enorme Abkürzung möglich    | Erkennung muss robust sein               |
|         5 | **Glyph Cache + Font Pipeline**        | Office-PDFs profitieren stark                              | PDF-Fontmodell ist komplex               |
|         6 | **Streaming Decode/Encode**            | Weniger Kopien, weniger RAM, bessere Pipeline              | Nicht alle Filter gut streambar          |
|         7 | **Optional Hybrid GPU**                | Stark bei Vektor/Transparenz/high DPI                      | Readback + PNG können Vorteil auffressen |
|         8 | **Color/Transparency Fast Paths**      | Opaque Normal-Blend ist häufig; exotisch langsam behandeln | Korrektheit bei Prepress-PDFs schwer     |

---

# Was ich **nicht** als Hauptstrategie wählen würde

**Zero Copy als Hauptclaim**
Gut, aber nicht differenzierend genug. Wegen Kompression, Farbraumkonvertierung und Rasterisierung entstehen ohnehin neue Daten.

**Nur Parser schneller machen**
PDF-Parsing ist wichtig, aber bei PNG-Rendering sind Raster, Bilder, Fonts, Blending und PNG-Encoding meist gewichtiger.

**GPU-only**
Für Server/CLI/Batch-PDF→PNG nicht immer ideal. Ohne GPU, bei kleinen Seiten oder bei bildlastigen PDFs kann CPU schneller und einfacher sein.

**Vollständige PDF-2.0-/Prepress-Korrektheit als erstes Ziel**
Das killt Geschwindigkeit und Fokus. Besser: realistische Dokumentklassen zuerst, harte PDF-Features isoliert nachziehen.

**Per-page Parallelism reicht**
Bestandslösungen können das bereits teilweise. Die Edge liegt eher in single-page tiling + Fast Paths.

---

# Konkrete Greenfield-Architektur

So würde ich es bauen:

```text
pdf-bytes
  ↓
Lazy object resolver / xref / stream decoder
  ↓
Page compiler
  - graphics state
  - resources
  - forms/xobjects
  - fonts
  - images
  ↓
Typed Page IR
  - DrawImage
  - FillPath
  - StrokePath
  - DrawGlyphRun
  - Clip
  - TransparencyGroup
  - Shading/Pattern
  ↓
Page classifier
  - image-only
  - text-heavy
  - vector-heavy
  - transparency-heavy
  - exotic/prepress
  ↓
Scheduler
  - direct-image path
  - CPU SIMD tiles
  - optional hybrid GPU
  ↓
PNG row/tile writer
```

Interne Formate:

```text
Framebuffer: premultiplied RGBA8, tile-local
Coverage: u8/u16 mask tiles
Paths: compact SoA geometry
Images: decoded lazily, preferably scanline/tile
Fonts: glyph mask/path cache
Resources: per-document cache keyed by object id + transform/DPI
```

---

# Realistische Performance-Edge

Global „schneller als MuPDF/PDFium für alles“ halte ich für unrealistisch. MuPDF, PDFium, Poppler und Ghostscript sind extrem gereift; MuPDF beschreibt sich selbst als schnellen, hochwertigen Renderer, und Ghostscript ist seit Langem als PDF/PostScript-Rasterizer im Einsatz. ([MuPDF.com][20]) ([ghostscript.readthedocs.io][21])

Realistisch wäre eher:

**1,5–3× schneller bei gescannten/OCR-PDFs**, wenn der Direct-Image-Pfad gut ist.
**1,2–2× schneller bei ARM64 für typische Office-/Text-/Image-Mischseiten**, wenn NEON-Compositing, Glyph Cache und Tiling sauber sind.
**Deutlich geringerer Peak-RAM**, eventuell 2–5×, wenn band-/tileweise gerendert und PNG gestreamt wird.
**Bessere Tail-Latency** bei riesigen Seiten, weil man nicht riesige Full-Page-Buffers und unnötige Objekte erzeugt.

Das wären aus meiner Sicht die plausiblen Zielmetriken, nicht „wir schlagen alles überall“.

---

# Benchmark-Setup, bevor man viel baut

Ich würde von Tag 1 an gegen diese Kategorien messen:

1. **Scanned/OCR**

   * 300 DPI Scan, JBIG2/CCITT/JPEG/JPEG2000
2. **Office**

   * Text, Bilder, einfache Vektoren
3. **Design**

   * Transparenz, Soft Masks, Gradients
4. **CAD/Maps**

   * extrem viele Pfade
5. **CJK/Fonts**

   * große Font- und Glyph-Komplexität
6. **Pathologische PDFs**

   * kaputte XRefs, Object Streams, riesige Images, ungewöhnliche Color Spaces

Metriken:

```text
ms/page
pages/sec
first page latency
peak RSS
allocations/page
decode time
IR compile time
raster time
PNG encode time
pixel diff vs reference renderer
```

Wichtig: MuPDF/PDFium/Ghostscript/Poppler nur als **Benchmark und Korrektheits-Orakel**, nicht als Dependency.

---

# Meine klare Empfehlung

Der stärkste Greenfield-Winkel wäre:

1. **CPU-first Renderer**
2. **ARM64/NEON-first Hotpath**
3. **Tile-/Band-basierte Pipeline**
4. **Typed, kompakte Display-List**
5. **Aggressiver Direct-Image-Fast-Path**
6. **Glyph-/Image-/Resource-Caches**
7. **Streaming PNG**
8. **Hybrid GPU erst später, datengetrieben**

Der Claim wäre dann nicht generisch „PDF Rendering in Rust“, sondern spezifischer:

> „A Rust-native PDF rasterizer optimized for high-throughput PDF-to-image conversion on modern ARM64 and server CPUs, with workload-aware fast paths for scanned, office, and vector-heavy PDFs.“

Das ist deutlich glaubwürdiger — und technisch auch eher dort, wo etablierte Universalrenderer angreifbar sind.

[1]: https://blog.adobe.com/en/publish/2022/01/31/20-years-of-transparency-in-pdf "20 years of transparency in PDF | Adobe"
[2]: https://mupdf.readthedocs.io/en/1.27.2/cookbook/c/multi-threaded.html "Multi-threaded - MuPDF 1.27.2"
[3]: https://mozilla.github.io/pdf.js/api/draft/module-pdfjsLib.html "JSDoc: Module: pdfjsLib"
[4]: https://docs.rs/hayro "hayro - Rust"
[5]: https://docs.rs/hayro-interpret "hayro_interpret - Rust"
[6]: https://doc.rust-lang.org/std/simd/index.html "std::simd - Rust"
[7]: https://docs.rs/rten-simd "rten_simd - Rust"
[8]: https://docs.rs/rpdfium-codec "rpdfium_codec - Rust"
[9]: https://docs.rs/pdf-syntax "pdf_syntax - Rust"
[10]: https://docs.rs/skrifa/latest/skrifa "skrifa - Rust"
[11]: https://developer.chrome.com/blog/memory-safety-fonts "Memory safety for web fonts  |  Blog  |  Chrome for Developers"
[12]: https://github.com/dfrg/swash "GitHub - dfrg/swash: Font introspection, complex text shaping and glyph rendering. · GitHub"
[13]: https://github.com/linebender/vello "GitHub - linebender/vello: A GPU compute-centric 2D renderer. · GitHub"
[14]: https://docs.rs/wgpu/ "wgpu - Rust"
[15]: https://docs.rs/vello_hybrid "vello_hybrid - Rust"
[16]: https://linebender.org/blog/tmil-19/ "Linebender in July 2025 - Linebender"
[17]: https://github.com/linebender/vello/issues/670 "Sparse strip path rendering · Issue #670 · linebender/vello · GitHub"
[18]: https://docs.rs/vello_common "vello_common - Rust"
[19]: https://docs.rs/zune-png "zune_png - Rust"
[20]: https://mupdf.com/ "MuPDF: The ultimate library for managing PDF documents"
[21]: https://ghostscript.readthedocs.io/en/latest/Use.html?utm_source=chatgpt.com "Using Ghostscript — Ghostscript 10.08.0 documentation"
