import { ArdoLink as Link } from "ardo/runtime";
import { MonoCredit, MonoPassage, MonoSection } from "ferramenta-family/monografie";

import { count, registryNote, samples, spelled } from "./home-data";
import { BudgetTable, OutcomeLedger, PreviewSpecimen } from "./preview-specimens";

/* The homepage's reading chapters: what a preview is, budgets, outcomes. */

const { defaults, buckets, outcomes, profiles } = samples;

export function Explainer() {
  return (
    <MonoSection id="overview" title="A small picture of a file you did not write.">
      <MonoPassage
        note={{
          term: "PDF",
          children:
            "A file format for fixed pages that look the same on every screen and printer. Office suites, browsers and scanners all write it.",
        }}
      >
        <p>
          When someone uploads a PDF, the application usually wants a small picture of its first
          page: in a file list, beside an invoice, before anyone opens it. Drawing that picture
          means reading the whole format, with its fonts, images, vector drawings, compression,
          forms and encryption. And the file comes from someone you do not know.
        </p>
      </MonoPassage>
      <MonoPassage
        note={{
          side: "right",
          term: "Untrusted input",
          children:
            "A file from outside your control. It may be broken by accident, or built on purpose to make a program crash, hang or run out of memory.",
        }}
      >
        <p>
          Ferrugo is a Rust library and command-line tool for this one job. It reads one page, draws
          it into an image no larger than you ask for, and stops at limits you set for time and
          memory. It needs no PDFium, the renderer inside Chrome, and no other renderer installed
          beside it. Below is the first page of a scanned patent from the repository’s test files.
          Switch the size and the budget profile.
        </p>
      </MonoPassage>
      <PreviewSpecimen />
      <MonoCredit>
        <Link to="/guide/getting-started/">Render your first preview</Link>
      </MonoCredit>
    </MonoSection>
  );
}

export function Budgets() {
  const note = registryNote("A budget has a scope");
  return (
    <MonoSection
      id="budgets"
      title="Limits you choose, checked while it works."
      standfirst="Every render has a largest edge and a time limit. Behind them, a budget profile caps how much memory each part of the work may take."
    >
      <MonoPassage
        note={{
          term: "Budget",
          children:
            "An upper limit Ferrugo checks during a render. When the work would cross it, Ferrugo stops and says which kind of limit it was, instead of growing.",
        }}
      >
        <p>
          Unless you say otherwise, Ferrugo renders the first page, at most{" "}
          {count(defaults.maxEdge)} pixels on its longer side, within{" "}
          {spelled(defaults.timeoutSeconds)} seconds. A page that would need more is scaled down,
          and a render that takes longer stops with a timeout. Choose a smaller edge for a file list
          and a larger one for a preview panel.
        </p>
      </MonoPassage>
      <MonoPassage note={{ side: "right", term: note.title, children: note.body }}>
        <p>
          Two budget profiles come with the crate. The default suits a desktop tool; the low-memory
          profile is meant for servers and batch jobs that render many files side by side. A page
          raster under the low-memory profile holds at most{" "}
          {count(profiles.lowMemory.maxPagePixels)} pixels, enough for a thumbnail and not for a
          full preview, as the patent above shows. Both profiles are plain values, and you can set
          your own.
        </p>
      </MonoPassage>
      <BudgetTable />
      <MonoCredit>
        <Link to="/reference/native-backend/">Every budget and policy of the renderer</Link>
      </MonoCredit>
    </MonoSection>
  );
}

/** The public documents in the ledger, and what became of them. */
const publicIds = new Set(["patent", "chrome-print", "data-brief", "tax-form"]);
const publicOutcomes = outcomes.filter((entry) => publicIds.has(entry.id));
const refused = publicOutcomes.filter((entry) => !entry.default.rendered);
const refusedAs = (kind: string) => refused.filter((entry) => entry.default.class === kind).length;

export function Outcomes() {
  const note = registryNote("A preview has a narrower job");
  return (
    <MonoSection
      id="outcomes"
      title="When there is no preview, there is a reason."
      standfirst="Ferrugo answers every file with a picture or a named outcome. Your code decides what a visitor sees next."
    >
      <MonoPassage
        note={{
          term: "Bucket",
          children:
            "A stable name for the feature or limit that stopped a render, such as form.xfa-dynamic. You can count it, log it and explain it to a user.",
        }}
      >
        <p>
          An error you can route beats a message you have to parse. Every failed render belongs to
          one of six classes: encrypted, malformed, unsupported, timeout, cancelled or internal. An
          unsupported file also carries one of {spelled(buckets.length)} named buckets, so you can
          tell a memory budget from a dynamic form without reading the text.
        </p>
      </MonoPassage>
      <MonoPassage note={{ side: "right", term: note.title, children: note.body }}>
        <p>
          The table runs {spelled(outcomes.length)} files from the repository through both budget
          profiles: public documents, generated test files and reduced attack inputs. Read it as a
          status report. The attack inputs are refused the way they should be. Of the{" "}
          {spelled(publicOutcomes.length)} public documents, {spelled(refused.length)} do not
          preview yet: {spelled(refusedAs("unsupported"))} comes back as unsupported and{" "}
          {spelled(refusedAs("malformed"))} as malformed, although they are ordinary published
          files. That is where an early renderer stands, and why you should test it with your own
          files first.
        </p>
      </MonoPassage>
      <OutcomeLedger />
      <MonoCredit>
        <Link to="/reference/errors/">Every class and bucket, and how to handle it</Link>
      </MonoCredit>
    </MonoSection>
  );
}
