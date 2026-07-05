import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { dirname, resolve } from 'node:path'

import { inspect, render } from '../index.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
const fixture = resolve(__dirname, '../../../fixtures/generated/text-page.pdf')
const malformed = Buffer.from('not a pdf')

const input = await readFile(fixture)

const png = await render(input, { outputFormat: 'png', maxEdge: 160 })
assert.equal(png.outputFormat, 'png')
assert.equal(png.pixelFormat, 'png')
assert.equal(png.data.subarray(0, 8).toString('hex'), '89504e470d0a1a0a')
assert.ok(png.width > 0)
assert.ok(png.height > 0)

const rgba = await render(input, { outputFormat: 'rgba', maxEdge: 160 })
assert.equal(rgba.outputFormat, 'rgba')
assert.equal(rgba.pixelFormat, 'rgba8')
assert.equal(rgba.data.length, rgba.stride * rgba.height)

const metadata = await inspect(input)
assert.equal(metadata.pageCount, 1)
assert.ok(metadata.firstPage.width > 0)
assert.ok(metadata.firstPage.height > 0)

await assert.rejects(
  () => render(malformed, { outputFormat: 'png' }),
  (error) => error && error.code === 'malformed',
)

await assert.rejects(
  () => render(input, { outputFormat: 'png', timeoutMs: 0 }),
  (error) => error && error.code === 'timeout',
)

// Synchronous option-validation errors carry the same `code` shape as
// asynchronous render errors.
assert.throws(
  () => render(input, { maxEdge: 0 }),
  (error) => error && error.code === 'invalid-argument',
)
