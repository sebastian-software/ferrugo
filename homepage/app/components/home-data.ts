import { family } from "ferramenta-family/registry";

import homeSamples from "../data/home-samples.json";

/* Everything the homepage shows about Ferrugo's behavior is generated, never
   edited by hand: the previews in app/data/previews/ and the facts and
   outcomes in app/data/home-samples.json (scripts/render-home-samples.mjs),
   the release facts in app/data/site.json (scripts/write-site-data.mjs, at
   build time). */
export const samples = homeSamples;

/** The crate version that rendered the specimens, from crates.io. */
export const renderedWith = samples.renderedWith.ferrugo;

const previewFiles = import.meta.glob<string>("../data/previews/*.png", {
  eager: true,
  query: "?url",
  import: "default",
});
/** The bundled URL of a rendered preview, by the file name the samples record. */
export function previewUrl(file: string) {
  const url = previewFiles[`../data/previews/${file}`];
  if (url === undefined) throw new Error(`The preview ${file} is missing.`);
  return url;
}

const member = family.find((entry) => entry.name === "ferrugo");
if (!member) throw new Error("Ferrugo is missing from the family registry.");
export const ferrugo = member;

/** A registry note by its title, for the margin. */
export function registryNote(title: string) {
  const note = ferrugo.editorial?.notes.find((entry) => entry.title === title);
  if (!note) throw new Error(`The registry has no Ferrugo note "${title}".`);
  return note;
}

/** One recorded outcome by its id. */
export function outcome(id: string) {
  const found = samples.outcomes.find((entry) => entry.id === id);
  if (!found) throw new Error(`The samples have no outcome "${id}".`);
  return found;
}

/** A path under the site's base, for plain links that the router does not prefix. */
export const withBase = (path: string) => `${import.meta.env.BASE_URL}${path.replace(/^\//, "")}`;

const NUMBERS = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];
/** Small counts in words, as the reading text writes them. */
export const spelled = (amount: number) => NUMBERS[amount] ?? amount.toLocaleString("en-US");

/** The first letter in upper case, for a label that starts with a count. */
export const capitalized = (text: string) => `${text.charAt(0).toUpperCase()}${text.slice(1)}`;

/** The first letter in lower case, to set a registry phrase inside a sentence. */
export const lowerFirst = (text: string) => `${text.charAt(0).toLowerCase()}${text.slice(1)}`;

/** Bytes in binary megabytes, as the budgets are set. */
export const megabytes = (bytes: number) => `${String(bytes / 1024 / 1024)} MB`;

/** A count with thousands separators. */
export const count = (value: number) => value.toLocaleString("en-US");
