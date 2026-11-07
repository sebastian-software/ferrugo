/* eslint-disable security/detect-non-literal-fs-filename -- Only the fixed, repository-owned documents below are read. */
import { readFileSync } from "node:fs";

/** The site is served from the repository's GitHub Pages path. */
export const base = "/ferrugo/";
export const origin = "https://sebastian-software.github.io";

/**
 * The documents the site publishes, and where. The getting-started page is
 * written for the site; the others come unchanged from docs/.
 */
const published = [
  { source: "homepage/content/getting-started.md", path: "guide/getting-started" },
  { source: "docs/errors.md", path: "reference/errors", title: "Errors and outcomes" },
  {
    source: "docs/backend/native.md",
    path: "reference/native-backend",
    title: "Rust-native backend",
  },
  { source: "docs/benchmarks.md", path: "evidence/benchmarks", title: "Renderer benchmarks" },
];

export const documents = published.map(({ source, path, title }) => {
  const content = readFileSync(new URL(`../../${source}`, import.meta.url), "utf8");
  const frontmatter = content.startsWith("---\n") ? content.split("---\n")[1] : undefined;
  const heading =
    (frontmatter && /^title: (.+)$/m.exec(frontmatter)?.[1]) ??
    /^# (.+)$/m.exec(content)?.[1].replaceAll("`", "") ??
    source;
  return { source, path, title: title ?? heading, section: path.split("/")[0] };
});

/** Built pages, relative to the output directory (which already sits under the base). */
export const expectedPages = [
  "index.html",
  ...documents.map(({ path }) => `${path}/index.html`),
].sort();

/** The public URL path of a built page, under the base. */
export function pagePath(page) {
  return page === "index.html" ? base : `${base}${page.replace(/index\.html$/, "")}`;
}
