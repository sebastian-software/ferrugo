# Post-1.0 Unsupported Feature Triage 2026-06-26

Milestone: 0161.

## Decision

Keep the post-1.0 unsupported backlog small, explicit, and impact-ranked. The
current generated corpus has 8 typed unsupported rows across 5 buckets, plus 1
expected encrypted input. The top follow-up work should be chosen by document
family impact and implementation risk, not by trying to close the full PDF
specification surface.

The PDFium-free supported runtime slice remains intact: `browser-print`,
`office-export`, and `form` render 87/87 fixtures natively with 0 fallbacks and
0 errors. Unsupported triage must not reintroduce runtime PDFium fallback.

## Evidence

Supported corpus artifact: `target/triage-0161-supported-corpus-gate.json`

Unsupported classification artifact:
`target/triage-0161-unsupported-classification.json`

| Scope | Total | Native rendered | Fallback required | Errors |
| --- | ---: | ---: | ---: | ---: |
| Core supported families | 87 | 87 | 0 | 0 |
| Full corpus | 186 | 177 | 8 | 1 encrypted |

## Unsupported Bucket Ranking

| Rank | Bucket | Count | Families | Render impact | Implementation risk | Owner route |
| ---: | --- | ---: | --- | --- | --- | --- |
| 1 | `image.filter` | 3 | `scan` | High for scanner, fax, archive, and compliance workflows where a page can be entirely image-based. | High: JPX and JBIG2 need careful decoder, isolation, fuzz, and memory decisions. | 0209 codec policy; 0170 image-heavy memory gate. |
| 2 | `graphics.transparency` | 2 | `report` | Medium to high for reports, dashboards, presentations, and design exports with advanced blends or soft masks. | High: compositing semantics and intermediate surfaces can affect memory and visual parity. | 0183 mixed transparency; 0213 transparency memory. |
| 3 | `graphics.optional-content` | 1 | `presentation` | Medium for layered presentations, maps, CAD-style overlays, and generated layer exports. | Medium: policy/UI state has to be explicit, but the current frequency is low. | 0192 optional-content state and layer flattening. |
| 4 | `graphics.pattern-shading` | 1 | `report` | Low to medium in the current corpus; higher for print/design/vector-heavy exports. | Medium to high: mesh/pattern fidelity can expand quickly without reductions. | 0166 office vector effects; 0204 chart/vector effects. |
| 5 | `form.xfa-dynamic` | 1 | `mixed-layout` | Medium for legacy enterprise/government forms; low for static AcroForm workflows. | High: dynamic XFA implies a runtime model outside current renderer scope. | 0206 form appearance/flattening policy; keep dynamic XFA as boundary. |

Encrypted input remains an expected `encrypted` policy error, not an unsupported
feature bucket.

## Bucket Details

### `image.filter`

Fixtures:

- `unsupported-ccitt-image.pdf`
- `unsupported-jbig2-image.pdf`
- `unsupported-jpx-image.pdf`

Current behavior: all three rows return typed `image.filter` fallback in the
`scan` family.

Next slice: decide the Rust-native deployment policy for CCITT, JPX, and JBIG2
before adding decoders. The first implementation slice should prefer the most
common scan workflow with bounded memory and fuzz coverage. Do not add an unsafe
or network-fetched codec path.

Validation gate: codec-focused fallback summary, image-heavy benchmark, memory
budget profile, fuzz smoke, package profile checks.

### `graphics.transparency`

Fixtures:

- `extgstate-luminosity-soft-mask.pdf`
- `unsupported-blend-mode.pdf`

Current behavior: both rows return typed `graphics.transparency` fallback in the
`report` family. Supported alpha, group, image-soft-mask, Multiply, and Screen
fixtures already have native paths.

Next slice: keep unsupported advanced blend and luminosity soft-mask behavior
separate from supported transparency optimization. Start with reduced fixtures
that isolate one blend or mask semantic at a time, then measure intermediate
surface memory.

Validation gate: transparency fixture visual comparison, native-only tests,
memory profile for intermediate surfaces, low-memory thumbnail smoke.

### `graphics.optional-content`

Fixture:

- `optional-content-ocmd.pdf`

Current behavior: the row returns typed `graphics.optional-content` fallback in
the `presentation` family. Simple default-on/default-off OCG fixtures render
natively.

Next slice: define OCMD membership and flattening semantics before expanding UI
or producer coverage. The default runtime should continue to avoid PDFium
fallback and expose the typed boundary.

Validation gate: optional-content fallback summary, layer fixture visual diff,
metadata/API snapshot for exposed layer state.

### `graphics.pattern-shading`

Fixture:

- `mesh-shading-unsupported.pdf`

Current behavior: the row returns typed `graphics.pattern-shading` fallback in
the `report` family. The supported Type 4 mesh fixture is separate and should
remain in the accepted/native path.

Next slice: keep mesh/pattern failures reduction-based. Prefer office/chart
fixtures that prove common exported vector effects rather than implementing
every shading type at once.

Validation gate: office vector visual subset, shading/pattern unit tests,
vector stress benchmark.

### `form.xfa-dynamic`

Fixture:

- `xfa-dynamic-no-static-appearance.pdf`

Current behavior: the row returns typed `form.xfa-dynamic` fallback in the
`mixed-layout` family. Static XFA with AcroForm appearance remains supported.

Next slice: keep dynamic XFA outside native renderer execution unless a later
product decision explicitly scopes a safe flattening model. For 1.x, preserve
the typed user-facing boundary and improve guidance for consumers.

Validation gate: static/dynamic XFA fallback summary, form appearance visual
subset, API diagnostics snapshot.

## Triage Report Format

Every newly observed unsupported case should be reported with these fields:

| Field | Required | Meaning |
| --- | --- | --- |
| `artifact` | yes | CLI JSON, corpus manifest, or issue attachment that introduced the case. |
| `fixture_or_source` | yes | Committed fixture path, private corpus identifier, or producer/source label without private data. |
| `bucket` | yes | Stable typed unsupported bucket such as `image.filter`. |
| `family` | yes | Document family affected: scan, report, presentation, form, office-export, browser-print, or mixed-layout. |
| `frequency` | yes | Count and denominator in the measured corpus or producer subset. |
| `render_impact` | yes | User-visible impact: blank page, missing layer, missing image, degraded visual, policy-only. |
| `implementation_risk` | yes | Memory, security, dependency, correctness, or API risk. |
| `owner_route` | yes | Next milestone or backlog slice that owns the decision. |
| `runtime_pdfium` | yes | Must stay `not required`; PDFium may only be maintainer oracle context. |
| `validation_gate` | yes | Small command set that proves the case stays typed or becomes supported. |

Reports may include historical PDFium output as archived context, but a
consumer-facing mitigation must not depend on runtime PDFium.

## Post-1.0 Implementation Backlog

1. Codec deployment policy and first safe scan-codec slice: closes the highest
   family impact (`scan`, 3 rows) and forces security/memory decisions before
   implementation.
2. Transparency edge-case reductions: keep advanced blends and soft-mask
   semantics isolated while preserving memory budgets.
3. Optional-content membership policy: unblock layered presentation/map cases
   without adding viewer UI scope prematurely.
4. Mesh/pattern shading reductions for common office/chart exports: handle the
   observed report gap through small vector-effect fixtures.
5. Dynamic XFA guidance and diagnostics: keep the unsupported boundary stable
   unless a future product decision scopes flattening.

## Validation

Commands run:

```sh
cargo test --workspace --no-default-features
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --include-family browser-print --include-family office-export --include-family form --fail-on-fallback --max-edge 160 --output target/triage-0161-supported-corpus-gate.json
cargo run -p ferrugo-cli --no-default-features -- summarize-fallbacks fixtures/generated --manifest fixtures/corpus-manifest.tsv --max-edge 160 --output target/triage-0161-unsupported-classification.json
```
