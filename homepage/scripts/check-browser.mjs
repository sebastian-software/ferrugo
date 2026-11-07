/* eslint-disable security/detect-non-literal-fs-filename -- Vite and axe paths are package-owned; pages come from the site route list. */
import assert from "node:assert/strict";
import { mkdir, readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { preview } from "vite";

import { base, expectedPages, pagePath } from "./site-pages.mjs";

const homepageRoot = fileURLToPath(new URL("..", import.meta.url));
const require = createRequire(import.meta.url);
const axeSource = await readFile(require.resolve("axe-core/axe.min.js"), "utf8");
const requestedPort = Number(process.env.FERRUGO_PREVIEW_PORT ?? 0);
assert(
  Number.isInteger(requestedPort) && requestedPort >= 0 && requestedPort <= 65_535,
  "FERRUGO_PREVIEW_PORT must be a valid TCP port",
);
const selectors = {
  nav: ".mono-header .mono-menu nav",
  // Documentation pages carry Ardo's drawer; the homepage folds its links into the Monografie menu.
  menu: 'button[aria-label="Toggle menu"]',
  homeMenu: ".mono-header .mono-menu-button",
  flyout: '[role="dialog"]',
  search: 'button[aria-label="Search"]',
};

let browser;
let previewServer;
try {
  previewServer = await preview({
    configFile: false,
    root: homepageRoot,
    logLevel: "silent",
    base,
    build: { outDir: "build/client" },
    preview: {
      host: "127.0.0.1",
      port: requestedPort,
      strictPort: requestedPort !== 0,
    },
  });
  const address = previewServer.httpServer.address();
  assert(
    address && typeof address !== "string",
    "Vite preview did not expose a listening TCP address",
  );
  const origin = `http://127.0.0.1:${address.port}`;
  /** A page of the site, under the base path GitHub Pages serves it from. */
  const at = (path) => new URL(`${base}${path.replace(/^\//, "")}`, origin).href;
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ colorScheme: "light" });
  const runtimeErrors = [];
  page.on("pageerror", (error) => runtimeErrors.push(error.message));
  for (const pageFile of expectedPages) {
    for (const viewport of [
      { name: "phone-small", width: 320, height: 760 },
      { name: "phone-compact", width: 375, height: 812 },
      { name: "phone", width: 390, height: 844 },
      { name: "phone-wide", width: 640, height: 900 },
      { name: "tablet", width: 768, height: 1024 },
      { name: "landscape", width: 844, height: 390 },
      { name: "tablet-wide", width: 1024, height: 900 },
      { name: "desktop", width: 1440, height: 900 },
    ]) {
      await verifyViewport({ page, pageFile, viewport, origin });
    }
  }

  await verifyHomepage(page, at);
  await verifyNavigation(page, at);
  await verifySearch(page, at);
  await page.emulateMedia({ reducedMotion: "reduce" });
  const motion = await page
    .locator(".mono, .mono *")
    .evaluateAll((elements) =>
      elements
        .map((element) => getComputedStyle(element).animationDuration)
        .filter((value) => value !== "0s"),
    );
  assert.deepEqual(motion, [], "Reduced motion still animates the page");
  assert.deepEqual(runtimeErrors, [], "The site raised browser runtime errors");
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await capture(page, at);
  console.log(
    "Verified all pages at eight widths, the homepage's real-output toggles, ledger and menu, navigation, search, focus return, reduced motion and runtime errors.",
  );
} finally {
  await browser?.close();
  await previewServer?.close();
}

/** The homepage's real-output toggles, the outcome ledger and the Monografie menu. */
async function verifyHomepage(page, at) {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(at("/"), { waitUntil: "load" });
  await verifyPreview(page);
  await verifyLedger(page);
  await verifyHomepageMenu(page, at);
}
async function verifyPreview(page) {
  const section = page.locator("#overview");
  const preview = section.locator(".fg-preview img");
  await preview.waitFor();
  assert.equal(
    await preview.evaluate((image) => image.naturalWidth),
    436,
    "The preview did not load",
  );
  await section.getByRole("button", { name: "Low-memory budgets", exact: true }).click();
  await section.getByText("No preview, and the reason", { exact: true }).waitFor();
  assert.match(await section.textContent(), /renderer\.memory-budget/);
  await section.getByRole("button", { name: "160 px", exact: true }).click();
  await section.getByText("109 × 160 pixels, shown at actual size", { exact: true }).waitFor();
  assert.equal(
    await preview.evaluate((image) => image.naturalWidth),
    109,
    "The small preview did not load",
  );
}
async function verifyLedger(page) {
  const ledger = page.locator("#outcomes table");
  assert.equal(await ledger.locator("tbody tr").count(), 9, "The outcome ledger lost a row");
  assert.match(await ledger.textContent(), /form\.xfa-dynamic/);
  assert.match(await ledger.textContent(), /encrypted/);
  assert.doesNotMatch(await page.locator("main").textContent(), /NaN|Infinity|undefined/);
}
async function verifyHomepageMenu(page, at) {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(at("/"), { waitUntil: "load" });
  const button = page.locator(selectors.homeMenu);
  await button.click();
  await page.locator(selectors.nav).getByRole("link", { name: "Guide", exact: true }).waitFor();
  await page.keyboard.press("Escape");
  assert.equal(await button.getAttribute("aria-expanded"), "false", "The menu did not close");
  assert(
    await button.evaluate((element) => element === document.activeElement),
    "Menu focus was not restored",
  );
}
async function verifyNavigation(page, at) {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(at("/guide/getting-started/"), { waitUntil: "load" });
  await page.getByRole("heading", { name: "Getting started", level: 1, exact: true }).waitFor();
  const menu = page.locator(selectors.menu);
  await focusMenuByKeyboard(page, menu);
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").waitFor();
  await page.keyboard.press("Escape");
  await page.getByRole("dialog").waitFor({ state: "detached" });
  assert(
    await menu.evaluate((element) => element === document.activeElement),
    "Menu focus was not restored",
  );
  await page.keyboard.press("Enter");
  await page.getByRole("dialog").getByRole("link", { name: "Reference", exact: true }).click();
  await page.waitForURL((url) => url.pathname.startsWith(`${base}reference/errors`));
  await verifyLandscape(page, at);
}
async function focusMenuByKeyboard(page, menu) {
  let reached = false;
  for (let i = 0; i < 40; i++) {
    await page.keyboard.press("Tab");
    if (await menu.evaluate((element) => element === document.activeElement)) {
      reached = true;
      break;
    }
  }
  assert(reached, "Keyboard navigation does not reach the menu");
}
async function verifyLandscape(page, at) {
  await page.setViewportSize({ width: 844, height: 390 });
  await page.goto(at("/reference/errors/"), { waitUntil: "load" });
  const table = page
    .locator('div[role="region"][tabindex="0"]')
    .filter({ has: page.locator("table") })
    .first();
  await table.focus();
  assert(
    await table.evaluate((element) => element === document.activeElement),
    "Markdown tables are not keyboard focusable",
  );
  await page.locator(selectors.menu).focus();
  await page.keyboard.press("Enter");
  const lastLink = page.getByRole("dialog").getByRole("link").last();
  await lastLink.focus();
  const bounds = await lastLink.boundingBox();
  assert(
    bounds && bounds.y >= 0 && bounds.y + bounds.height <= 390,
    "Landscape menu link leaves the viewport",
  );
  await page.keyboard.press("Escape");
}
async function verifySearch(page, at) {
  for (const width of [320, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(at("/"), { waitUntil: "load" });
    const search = page.locator(selectors.search);
    await search
      .locator("kbd")
      .first()
      .evaluate((element) => {
        element.textContent = "Ctrl";
      });
    await search.hover();
    const hint = await search.locator('span[aria-hidden="true"]').boundingBox();
    assert(hint && hint.x >= 0 && hint.x + hint.width <= width, "Search hint leaves the viewport");
    await search.click();
    const dialog = page.getByRole("dialog");
    await dialog.locator("input").fill("unsupported feature buckets");
    await dialog.locator(`a[href*="${base}reference/errors"]`).first().waitFor();
    await page.keyboard.press("Escape");
    await dialog.waitFor({ state: "detached" });
    assert(
      await search.evaluate((element) => element === document.activeElement),
      "Search focus was not restored",
    );
  }
}
/** Full-page captures for review, outside the repository. */
async function capture(page, at) {
  const directory = process.env.FERRUGO_REVIEW_DIR ?? join(tmpdir(), "ferrugo-review");
  await mkdir(directory, { recursive: true });
  for (const [path, label] of [
    ["/", "home"],
    ["/guide/getting-started/", "guide"],
    ["/reference/errors/", "reference"],
  ]) {
    for (const width of [390, 1440]) {
      await page.setViewportSize({ width, height: 1000 });
      await page.goto(at(path), { waitUntil: "load" });
      await page.screenshot({ path: join(directory, `${label}-${width}.png`), fullPage: true });
    }
  }
}

async function verifyViewport({ page, pageFile, viewport, origin }) {
  await page.setViewportSize({ width: viewport.width, height: viewport.height });
  await page.goto(new URL(pagePath(pageFile), origin).href, { waitUntil: "load" });
  const layout = await page.evaluate(() => ({
    width: globalThis.document.documentElement.clientWidth,
    pageWidth: Math.max(
      globalThis.document.documentElement.scrollWidth,
      globalThis.document.body.scrollWidth,
    ),
  }));
  assert(
    layout.pageWidth <= layout.width + 1,
    `${pageFile} has horizontal page overflow at ${viewport.name} width ${viewport.width}px: ${layout.pageWidth}px`,
  );

  await verifyHeader(page, pageFile, viewport);
  await verifyAccessibility(page, pageFile, viewport);
  console.log(`Verified ${pageFile} at ${viewport.name} (${viewport.width}px).`);
}
/** The display of the first element a selector finds, or "none" when there is none. */
async function display(page, selector) {
  const element = page.locator(selector).first();
  if ((await element.count()) === 0) return "none";
  return element.evaluate((node) => getComputedStyle(node).display);
}
async function verifyHeader(page, pageFile, viewport) {
  // The Monografie header folds its links into a menu below 961px. On the
  // homepage the menu button takes them; documentation pages hide it and keep
  // Ardo's drawer, which also stands in for the guide sidebar while it is hidden.
  assert(
    await page.locator(selectors.search).isVisible(),
    "The site should keep search visible at each checked viewport",
  );
  const wide = viewport.width >= 961;
  const links = await display(page, ".mono-header .mono-menu");
  if (wide) assert.notEqual(links, "none", `${pageFile} hides the header links on wide screens`);
  else assert.equal(links, "none", `${pageFile} shows the header links at ${viewport.name}`);
  const homeMenu = await display(page, selectors.homeMenu);
  if (pageFile === "index.html") {
    assert.equal(homeMenu === "none", wide, `${pageFile} menu button at ${viewport.name}`);
    return;
  }
  assert.equal(homeMenu, "none", `${pageFile} shows a second menu at ${viewport.name}`);
  const drawer = await display(page, selectors.menu);
  const sidebar = await display(page, ".mono-reading-navigation");
  assert(
    drawer !== "none" || sidebar !== "none",
    `${pageFile} offers no documentation navigation at ${viewport.name}`,
  );
}
async function verifyAccessibility(page, pageFile, viewport) {
  // Colors are measured once running transitions (a code block's copy label
  // fades in after a resize) have settled.
  await page.evaluate(async () => {
    await Promise.all(
      globalThis.document.getAnimations().map(async (animation) => animation.finished),
    );
  });
  await page.addScriptTag({ content: axeSource });
  const results = await page.evaluate(async () =>
    globalThis.axe.run(globalThis.document, {
      runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"] },
    }),
  );
  if (results.violations.length === 0) return;
  const details = results.violations
    .map((violation) => {
      const targets = violation.nodes
        .map((node) => `  ${node.target.join(", ")}: ${node.failureSummary}`)
        .join("\n");
      return `${violation.id} (${violation.impact}): ${violation.help}\n${targets}`;
    })
    .join("\n");
  throw new Error(`Accessibility violations on ${pageFile} at ${viewport.name} width:\n${details}`);
}
