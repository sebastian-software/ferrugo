/* eslint-disable security/detect-non-literal-fs-filename -- Source and output paths come from the repository-owned document manifest. */
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { posix } from "node:path";
import { format } from "oxfmt";

import { documents } from "./site-pages.mjs";

const repository = new URL("../../", import.meta.url);
const routes = new URL("../app/routes/", import.meta.url);
const data = new URL("../app/data/", import.meta.url);
const destinations = new Map(documents.map(({ source, path }) => [source, `/${path}/`]));
const repoUrl = "https://github.com/sebastian-software/ferrugo";
await mkdir(data, { recursive: true });
for (const section of ["guide", "reference", "evidence"]) {
  await rm(new URL(`${section}/`, routes), { recursive: true, force: true });
}

for (const document of documents) {
  const source = await readFile(new URL(document.source, repository), "utf8");
  const body = source.startsWith("---\n") ? source.slice(source.indexOf("\n---\n") + 5) : source;
  const content = rewriteLinks(
    body.replaceAll("\r\n", "\n").replace(/^# .+\n/, ""),
    document.source,
  );
  const destination = new URL(`${document.path}.md`, routes);
  await mkdir(new URL(".", destination), { recursive: true });
  await writeFile(destination, `---\ntitle: ${JSON.stringify(document.title)}\n---\n${content}`);
}

// The release comes from release-please's manifest, the minimum Rust version
// from the workspace manifest: the only places either is declared.
const releases = JSON.parse(
  await readFile(new URL(".release-please-manifest.json", repository), "utf8"),
);
const version = releases["crates/ferrugo-cli"];
if (!version) throw new Error("The release manifest has no ferrugo version.");
const cargo = await readFile(new URL("Cargo.toml", repository), "utf8");
const rustVersion = /^rust-version = "([^"]+)"$/m.exec(cargo)?.[1];
if (!rustVersion) throw new Error("Cargo.toml must declare the minimum supported Rust version.");
const license = /^license = "([^"]+)"$/m.exec(cargo)?.[1];
if (!license) throw new Error("Cargo.toml must declare the workspace license.");
await writeData("site.json", { version, rustVersion, license });

const labels = { guide: "Guide", reference: "Reference", evidence: "Evidence" };
await writeData(
  "navigation.json",
  Object.entries(labels).map(([id, label]) => {
    const pages = documents
      .filter((document) => document.section === id)
      .map(({ path, title }) => ({ title, to: `/${path}/` }));
    if (pages.length === 0) throw new Error(`The ${label} section has no pages.`);
    return { id, label, to: pages[0].to, pages };
  }),
);

async function writeData(name, value) {
  const result = await format(name, JSON.stringify(value));
  if (result.errors.length) throw new Error(`Cannot format ${name}`);
  await writeFile(new URL(name, data), result.code);
}

/** Links between published documents stay on the site; every other link leads to GitHub. */
function rewriteLinks(markdown, source) {
  let fence;
  return markdown
    .split("\n")
    .map((line) => {
      const delimiter = /^\s*(`{3,}|~{3,})/.exec(line);
      if (delimiter) {
        if (!fence)
          fence = {
            character: delimiter[1][0],
            length: delimiter[1].length,
            rust: line.slice(delimiter[0].length).trim() === "rust",
          };
        else if (delimiter[1][0] === fence.character && delimiter[1].length >= fence.length)
          fence = undefined;
        return line;
      }
      if (fence) return fence.rust && /^#(?: |$)/.test(line) ? null : line;
      return line.replaceAll(/(?<=\]\()[^\s)]+/g, (href) => resolveLink(href, source));
    })
    .filter((line) => line !== null)
    .join("\n");
}

function resolveLink(href, source) {
  if (/^(?:[a-z][a-z0-9+.-]*:|\/|#)/i.test(href)) return href;
  const [file, fragment] = href.split("#");
  const target = posix.normalize(posix.join(posix.dirname(source), file));
  const destination = destinations.get(target);
  if (destination) return `${destination}${fragment ? `#${fragment}` : ""}`;
  const kind = file.endsWith("/") || !posix.extname(file) ? "tree" : "blob";
  return `${repoUrl}/${kind}/main/${target}${fragment ? `#${fragment}` : ""}`;
}
