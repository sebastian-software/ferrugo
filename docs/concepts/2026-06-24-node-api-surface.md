# Node API Surface

Status: exploratory concept note, not an approved implementation spec.
Date: 2026-06-24.

Phase 0 note: Node-API is a future product surface, not part of the initial
thumbnail probe. Phase 0 starts with a Rust CLI and Rust library facade so the
PDFium source-build probe can be measured before npm packaging, prebuilt
binaries, or addon lifecycle decisions are made.

## Position

The Node.js package should be a thin product surface over the Rust API, not a
separate engine. Keep PDF parsing, rendering, memory ownership, cancellation,
and error classification in Rust. The Node layer should translate JavaScript
types to stable Rust inputs and return buffers, objects, streams, and promises
with minimal policy.

Node-API is the right native boundary because Node documents it as ABI-stable
across Node.js versions and independent from the underlying JavaScript runtime.
`napi-rs` is the likely implementation framework because it targets
precompiled Node.js addons in Rust and has broad platform support.

## Initial Package Shape

Package name placeholder: `@ferrugo/node`.

```ts
import { Document } from '@ferrugo/node'

const document = await Document.open(buffer, { password: undefined })
const page = document.page(0)

const image = await page.render({
  scale: 2,
  background: '#ffffff',
  format: 'rgba',
})

console.log(document.pageCount)
console.log(page.width, page.height)
```

## Proposed JavaScript API

```ts
type OpenOptions = {
  password?: string
  repair?: 'auto' | 'strict' | 'always'
}

type RenderOptions = {
  scale?: number
  width?: number
  height?: number
  rotation?: 0 | 90 | 180 | 270
  background?: string | [number, number, number, number]
  format?: 'rgba' | 'bgra' | 'png'
  annotations?: boolean
  timeoutMs?: number
  signal?: AbortSignal
}

type RenderedPage = {
  width: number
  height: number
  stride: number
  format: 'rgba' | 'bgra' | 'png'
  data: Uint8Array
}

class Document {
  static open(input: Uint8Array | ArrayBuffer | string, options?: OpenOptions): Promise<Document>
  readonly pageCount: number
  page(index: number): Page
  metadata(): Promise<Record<string, string>>
  close(): void
}

class Page {
  readonly index: number
  readonly width: number
  readonly height: number
  render(options?: RenderOptions): Promise<RenderedPage>
  text(): Promise<string>
}
```

## Binding Rules

- Heavy work returns `Promise` and runs off the JavaScript thread.
- JavaScript buffers passed to Rust are copied or held through a safe lifetime
  guard; do not retain VM-managed pointers without ownership.
- Rendered bitmap output should be returned as an externally owned ArrayBuffer
  only when Rust can safely transfer lifetime to the JavaScript finalizer.
- Cancellation should be best-effort at operator or tile boundaries, not by
  killing native threads.
- Errors should be typed and stable:
  - `PdfSyntaxError`
  - `PdfPasswordError`
  - `PdfUnsupportedFeatureError`
  - `PdfRenderError`
  - `PdfTimeoutError`
  - `PdfInternalError`
- The Node surface should not expose PDFium's global initialization model.
  Initialization should be implicit and scoped by module state.

## Threading

The Rust core should aim to make independent `Document` instances renderable in
parallel. Page rendering can later use internal tiling or worker scheduling, but
the first binding should only promise that multiple render calls can run without
blocking the JavaScript event loop.

If a subsystem is not thread-safe, isolate that constraint inside the Rust core
and avoid leaking it as a global Node package limitation.

## Packaging

The npm package should eventually ship prebuilt binaries for common platforms
once the core is useful enough and the PDFium/Rust backend strategy is clearer:

- macOS arm64 and x64.
- Linux glibc x64 and arm64.
- Linux musl x64 where feasible.
- Windows x64.

Because the long-term goal is pure Rust, the package should avoid bundling
PDFium or MuPDF unless a later product decision explicitly accepts that tradeoff.
During early development, a separate `@ferrugo/pdfium-oracle` or dev-only tool
may be useful for differential testing.

## Sources

- Node-API documentation: https://nodejs.org/api/n-api.html
- NAPI-RS: https://napi.rs/
