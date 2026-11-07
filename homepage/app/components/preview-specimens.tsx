import { useState } from "ardo/react";
import {
  MonoCaption,
  MonoFacing,
  MonoSegmented,
  MonoSheet,
  MonoWide,
} from "ferramenta-family/monografie";

import { count, megabytes, previewUrl, renderedWith, samples, spelled } from "./home-data";
import "./preview-specimens.css";

/* The homepage's specimens: every image, size and outcome here is real output
   of the published crates, from app/data/home-samples.json and the PNG files
   beside it. */

const { preview, profiles } = samples;
const fileName = (path: string) => path.slice(path.lastIndexOf("/") + 1);
const inches = (points: number) => (points / 72).toFixed(1);

type Result = {
  rendered: boolean;
  width?: number;
  height?: number;
  class?: string;
  bucket?: null | string;
  message?: string;
};

/** A refusal as the API reports it: class, bucket and the display text. */
function Refusal({ result }: { result: Result }) {
  return (
    <dl className="fg-refusal">
      <div>
        <dt>error.class()</dt>
        <dd>
          <code>{result.class}</code>
        </dd>
      </div>
      <div>
        <dt>error.unsupported_feature_bucket()</dt>
        <dd>
          <code>{result.bucket ?? "None"}</code>
        </dd>
      </div>
      <div>
        <dt>error.to_string()</dt>
        <dd>
          <code>{result.message}</code>
        </dd>
      </div>
    </dl>
  );
}

const PROFILES = [
  { value: "default", label: "Default budgets", call: "NativeBackend::new()" },
  { value: "lowMemory", label: "Low-memory budgets", call: "NativeBackend::low_memory()" },
];

/** What the input is, as Ferrugo's inspect call and the fixture list describe it. */
function SourceSheet() {
  return (
    <MonoSheet label="The file that arrived" meta={fileName(preview.input)}>
      <dl className="fg-facts">
        <div>
          <dt>What it is</dt>
          <dd>U.S. patent 6,289,801, a scanned public record</dd>
        </div>
        <div>
          <dt>Pages</dt>
          <dd>{preview.pages}</dd>
        </div>
        <div>
          <dt>First page</dt>
          <dd>
            {count(preview.pageWidth)} × {count(preview.pageHeight)} points, about{" "}
            {inches(preview.pageWidth)} × {inches(preview.pageHeight)} inches
          </dd>
        </div>
        <div>
          <dt>Written by</dt>
          <dd>{preview.producer}</dd>
        </div>
      </dl>
    </MonoSheet>
  );
}

type Render = (typeof preview.renders)[number];

/** The rendered page at its own pixel size, or the refusal in its place. */
function ResultSheet({ render, profile }: { render: Render; profile: string }) {
  const call = PROFILES.find((entry) => entry.value === profile)?.call ?? "";
  const result: Result =
    profile === "default"
      ? { rendered: true, width: render.width, height: render.height }
      : render.lowMemory;
  return (
    <MonoSheet
      label={result.rendered ? "The preview" : "No preview, and the reason"}
      meta={`${call}, max_edge ${String(render.maxEdge)}`}
    >
      <div className="fg-result" aria-live="polite">
        {result.rendered ? (
          <figure className="fg-preview">
            <img
              src={previewUrl(render.file)}
              width={render.width}
              height={render.height}
              alt="The first page of U.S. patent 6,289,801 as Ferrugo rendered it: the printed front page with its bibliographic columns, the abstract, and the patent’s drawing of a face-shaped stencil."
            />
            <figcaption>
              {render.width} × {render.height} pixels, shown at actual size
            </figcaption>
          </figure>
        ) : (
          <Refusal result={result} />
        )}
      </div>
    </MonoSheet>
  );
}

/** The scanned patent's first page at two sizes and under two budget profiles. */
export function PreviewSpecimen() {
  const [edge, setEdge] = useState("640");
  const [profile, setProfile] = useState("default");
  const render = preview.renders.find((entry) => String(entry.maxEdge) === edge);
  if (!render) throw new Error(`The samples have no ${edge}px preview.`);
  return (
    <MonoWide>
      <div className="fg-toggles">
        <MonoSegmented
          label="Largest edge"
          value={edge}
          options={preview.renders.map((entry) => ({
            value: String(entry.maxEdge),
            label: `${String(entry.maxEdge)} px`,
          }))}
          onChange={setEdge}
        />
        <MonoSegmented
          label="Budgets"
          value={profile}
          options={PROFILES.map(({ value, label }) => ({ value, label }))}
          onChange={setProfile}
        />
      </div>
      <MonoFacing className="fg-facing">
        <SourceSheet />
        <ResultSheet render={render} profile={profile} />
      </MonoFacing>
      <MonoCaption>
        Real output of ferrugo-thumbnail and ferrugo-native {renderedWith} from crates.io, rendering
        page one of a fixture in the repository. The reference numerals in the drawing belong to the
        patent. Under the low-memory budgets one page raster may hold at most{" "}
        {count(profiles.lowMemory.maxPagePixels)} pixels, so the larger preview is refused before it
        is drawn.
      </MonoCaption>
    </MonoWide>
  );
}

/** What each input of the outcome ledger is, in words. */
const INPUTS: Record<string, { what: string; kind: string }> = {
  patent: { what: "A scanned U.S. patent", kind: "Public document" },
  "chrome-print": { what: "A government web page printed from Chrome", kind: "Public document" },
  "data-brief": { what: "A CDC data brief with charts", kind: "Public document" },
  "tax-form": { what: "IRS Form W-9, an XFA form with scripts", kind: "Public document" },
  invoice: { what: "A dense invoice page", kind: "Generated test file" },
  encrypted: { what: "A password-protected file", kind: "Generated test file" },
  "dynamic-form": { what: "A dynamic XFA form with no static page", kind: "Generated test file" },
  truncated: { what: "A file whose header is cut off", kind: "Reduced attack input" },
  "huge-image": {
    what: "A tiny page that declares an enormous image",
    kind: "Reduced attack input",
  },
};

const publicDocuments = samples.outcomes.filter(
  (entry) => INPUTS[entry.id]?.kind === "Public document",
);
const publicPreviews = publicDocuments.filter((entry) => entry.default.rendered).length;

function Outcome({ result }: { result: Result }) {
  if (result.rendered) {
    return (
      <>
        Preview,{" "}
        <span className="fg-size">
          {result.width} × {result.height}
        </span>
      </>
    );
  }
  return (
    <code>
      {result.class}
      {result.bucket === undefined || result.bucket === null ? "" : ` · ${result.bucket}`}
    </code>
  );
}

type Entry = (typeof samples.outcomes)[number];

function LedgerRow({ entry }: { entry: Entry }) {
  const input = INPUTS[entry.id];
  if (!input) throw new Error(`Describe the input "${entry.id}".`);
  return (
    <tr>
      <th scope="row">
        {input.what}
        <small>
          {input.kind}, <code>{fileName(entry.input)}</code>
        </small>
      </th>
      <td data-rendered={entry.default.rendered ? "" : undefined}>
        <Outcome result={entry.default} />
      </td>
      <td data-rendered={entry.lowMemory.rendered ? "" : undefined}>
        <Outcome result={entry.lowMemory} />
      </td>
    </tr>
  );
}

/** Every recorded input with what the render returned under both budget profiles. */
export function OutcomeLedger() {
  return (
    <MonoWide>
      <div
        className="fg-scroll"
        role="region"
        aria-label="What Ferrugo returned for each input"
        tabIndex={0}
      >
        <table className="mono-table fg-ledger">
          <thead>
            <tr>
              <th scope="col">Input</th>
              <th scope="col">Default budgets</th>
              <th scope="col">Low-memory budgets</th>
            </tr>
          </thead>
          <tbody>
            {samples.outcomes.map((entry) => (
              <LedgerRow key={entry.id} entry={entry} />
            ))}
          </tbody>
        </table>
      </div>
      <MonoCaption>
        Real output of ferrugo {renderedWith} from crates.io: page one of each fixture at the
        default largest edge of {count(samples.defaults.maxEdge)} pixels. The{" "}
        {spelled(publicDocuments.length)} public documents are the repository’s real-world set;{" "}
        {spelled(publicPreviews)} of them {publicPreviews === 1 ? "previews" : "preview"} with this
        release.
      </MonoCaption>
    </MonoWide>
  );
}

const BUDGETS = [
  { key: "maxPagePixels", label: "Pixels in one page raster", value: count },
  { key: "maxImageBytes", label: "Decoded bytes of one image", value: megabytes },
  { key: "maxTotalImageBytes", label: "Decoded image bytes on one page", value: megabytes },
  { key: "maxDisplayItems", label: "Drawing operations on one page", value: count },
] as const;

/** A few of the budgets each profile sets, read from the crate itself. */
export function BudgetTable() {
  return (
    <MonoWide>
      <div className="fg-scroll" role="region" aria-label="Render budgets" tabIndex={0}>
        <table className="mono-table fg-budgets">
          <thead>
            <tr>
              <th scope="col">Budget</th>
              <th scope="col" className="mono-num">
                <code>new()</code>
              </th>
              <th scope="col" className="mono-num">
                <code>low_memory()</code>
              </th>
            </tr>
          </thead>
          <tbody>
            {BUDGETS.map(({ key, label, value }) => (
              <tr key={key}>
                <th scope="row">{label}</th>
                <td className="mono-num">{value(profiles.default[key])}</td>
                <td className="mono-num">{value(profiles.lowMemory[key])}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <MonoCaption>
        From <code>NativeBackend::render_limits()</code> in ferrugo-native {renderedWith}. Each
        profile sets more budgets than these four, for fonts, text, patterns and transparency.
      </MonoCaption>
    </MonoWide>
  );
}
