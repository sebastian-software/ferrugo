import type { ArdoMetaFunction } from "ardo/runtime";

import { Fragment } from "ardo/react";
import { ArdoLink as Link } from "ardo/runtime";
import {
  MonoAction,
  MonoClosing,
  MonoColophon,
  MonoEngraving,
  MonoOpening,
} from "ferramenta-family/monografie";
import { WORKSHOP } from "ferramenta-family/registry";

import { count, ferrugo, lowerFirst, samples, withBase } from "../components/home-data";
import { Fit, Status } from "../components/home-engine";
import { Budgets, Explainer, Outcomes } from "../components/home-reading";
import { license, rustVersion, version } from "../data/site.json";

// oxlint-disable-next-line react/only-export-components -- Ardo reads route metadata.
export const meta: ArdoMetaFunction = () => [
  { title: "Ferrugo — Previews of PDFs you did not write." },
  {
    name: "description",
    content:
      "A PDF preview renderer for Rust. Ferrugo renders one page of an untrusted PDF into a small image under limits for size, time and memory, without PDFium, and names the reason when it cannot. Early, and candid about it.",
  },
];
// oxlint-disable-next-line react/only-export-components -- Ardo reads the route layout.
export const handle = { layout: "bare" };

const releaseLine = version.split(".").slice(0, 2).join(".");
const { defaults, buckets } = samples;

function Actions() {
  return (
    <>
      <MonoAction
        href={withBase("/guide/getting-started/#install-the-command-line")}
        primary
        detail="cargo install ferrugo --locked"
      >
        Start with the command line
      </MonoAction>
      <MonoAction
        href={withBase("/guide/getting-started/#use-the-rust-api")}
        detail="cargo add ferrugo-thumbnail ferrugo-native"
      >
        Use it from Rust
      </MonoAction>
    </>
  );
}

function Opening() {
  return (
    <MonoOpening
      product="ferrugo"
      instrumentLabel="Ferrugo's instrument: on the left, a large steel frame holds an ivory source page between two columns, with a rust-brown strip along its edge. Two rails and a screw-driven carriage lead to a smaller steel housing on the right, whose framed window shows the same page layout in miniature under a rust-brown stop."
      caption="An aperture turns a source document into a bounded preview."
      kicker={`Ferrugo ${releaseLine}, ${lowerFirst(ferrugo.what)} for Rust`}
      title="Previews of PDFs you did not write."
      subtitle="One page in, one small image out, within limits you set."
      standfirst={`Upload forms, inboxes and document stores fill up with PDFs from people you do not know. To show one in a list, you need a small picture of its first page, and drawing it means reading a large, old format that a broken or hostile file can turn against you. Ferrugo is a Rust library and command-line tool that renders that picture under explicit limits for size, time and memory, and names the reason when it cannot. ${ferrugo.audience}`}
      actions={<Actions />}
      facts={[
        {
          value: (
            <>
              <em>{buckets.length}</em> buckets
            </>
          ),
          label:
            "named reasons behind an unsupported outcome, from a missing feature to a spent budget",
        },
        { value: "No PDFium", label: "and no other renderer to install beside it" },
        {
          value: `${count(defaults.maxEdge)} px, ${String(defaults.timeoutSeconds)} s`,
          label: "the default largest edge and time limit for one page",
        },
        {
          value: `Rust ${rustVersion}+`,
          label: license.replace(" OR ", " or "),
        },
      ]}
    />
  );
}

// Keys are written out: minified builds do not keep function names.
const chapters = [
  { key: "explainer", Chapter: Explainer },
  { key: "budgets", Chapter: Budgets },
  { key: "outcomes", Chapter: Outcomes },
  { key: "status", Chapter: Status },
  { key: "fit", Chapter: Fit },
];

/** Where Ferrugo comes from, and who makes it. */
function Origin() {
  return (
    <section aria-label="Origin">
      <div className="mono-axis">
        <MonoColophon>
          {ferrugo.proof} It is one of the Ferramenta engines, made by {WORKSHOP.name} in{" "}
          {WORKSHOP.place}.
        </MonoColophon>
      </div>
    </section>
  );
}

export default function HomePage() {
  return (
    <>
      <Opening />
      {chapters.map(({ key, Chapter }, index) => (
        <Fragment key={key}>
          <MonoEngraving seed={index + 1} />
          <Chapter />
        </Fragment>
      ))}
      <MonoEngraving seed={chapters.length + 1} />
      <Origin />
      <div id="start">
        <MonoClosing
          title="Try it on your own files first."
          standfirst="Install the command line and render a few of the PDFs your users really send. Each outcome tells you whether Ferrugo can preview that kind of file today."
          actions={<Actions />}
          links={
            <>
              <a href="https://github.com/sebastian-software/ferrugo">GitHub repository</a>
              <Link to="/guide/getting-started/">Documentation</Link>
              <a href="https://docs.rs/ferrugo-thumbnail">API reference</a>
              <a href={WORKSHOP.consulting}>Integration and support</a>
            </>
          }
        />
      </div>
    </>
  );
}
