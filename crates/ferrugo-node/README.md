# ferrugo-node

Node-API bindings for Ferrugo native PDF thumbnails.

Install the npm package as `ferrugo`:

```sh
npm install ferrugo
```

The package exposes the stateless native backend only. It does not retain
`NativeDocumentSession` across JavaScript calls and does not bundle PDFium.

```js
import { readFile } from 'node:fs/promises'
import { render, inspect } from 'ferrugo'

const input = await readFile('fixtures/generated/text-page.pdf')
const image = await render(input, { outputFormat: 'png', maxEdge: 256 })
const metadata = await inspect(input)

console.log(image.width, image.height, image.data.length)
console.log(metadata.pageCount)
```

## API

- `render(input, options?)`: renders one page on the Node-API worker pool.
- `inspect(input)`: inspects document metadata on the Node-API worker pool.

`render` accepts the same bounded defaults as the Rust thumbnail facade. Set
`timeoutMs` to override the per-render timeout. JavaScript cancellation is not a
separate API surface in this first binding; use `timeoutMs` for deterministic
render cancellation.

Errors carry stable `code` values matching the Rust thumbnail error class:
`encrypted`, `malformed`, `unsupported`, `timeout`, `cancelled`, or `internal`.
Unsupported-feature errors also carry a `bucket` string. Synchronous option
validation failures use `code: "invalid-argument"`.

## Distribution

The published npm package is self-contained: it bundles the prebuilt
`ferrugo.<platform>.node` binaries for all supported targets and loads the
matching one at require time. There are no per-platform sub-packages and no
postinstall build step. The generated `index.js`/`index.d.ts` are regenerated
during the release workflow so they always match the released version.
