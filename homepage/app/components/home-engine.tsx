import { ArdoLink as Link } from "ardo/runtime";
import {
  MonoCredit,
  MonoLeadIn,
  MonoPassage,
  MonoSection,
  MonoTerms,
  MonoWide,
} from "ferramenta-family/monografie";

import { ferrugo, lowerFirst } from "./home-data";

/* The homepage's engine chapters: maturity and fit. */

const STATUS_TERMS = [
  {
    term: "Good fit today",
    description:
      "Preview thumbnails in a Rust service or a command-line job, with explicit memory and time budgets, for files you are ready to see refused.",
  },
  {
    term: "Not a fit yet",
    description:
      "A full PDF viewer, editing, signing, JavaScript or dynamic XFA forms, or a promise of the same picture every other renderer draws.",
  },
];

export function Status() {
  return (
    <MonoSection
      id="status"
      title="Early, and open about it."
      standfirst="Ferrugo is young. Its checks are broad, its renderer is not finished, and this page makes no speed claim yet."
    >
      <MonoPassage
        note={{
          term: "Golden image",
          children:
            "A stored reference render. A test draws the same file again and fails when the new picture drifts past a set threshold.",
        }}
      >
        <p>
          <MonoLeadIn>Every pull request</MonoLeadIn> runs formatting and lint checks, the test
          suite, a short fuzzing run that feeds the parser random and broken input, comparisons
          against golden images, checks that no PDFium code reaches the shipped packages, and a
          WebAssembly build. The open gaps in fidelity and features are tracked in the repository,
          not hidden behind a general claim of compatibility.
        </p>
      </MonoPassage>
      <MonoPassage
        note={{
          side: "right",
          term: "Reference renderer",
          children:
            "Another program that draws PDFs, used during development to compare pictures and timings. It is never part of Ferrugo at runtime.",
        }}
      >
        <p>
          The repository also records timing runs against reference renderers such as PDFium and
          Poppler, each with its machine, versions and caveats. Its own policy sets strict rules for
          a public speed claim: two stable runs, at least twenty samples per measurement, every
          version written down. This page waits for a run that compares several renderers side by
          side under those rules, so there is no speed chart here.
        </p>
      </MonoPassage>
      <MonoWide>
        <MonoTerms prose columns={2} items={STATUS_TERMS} />
      </MonoWide>
      <MonoCredit>
        <Link to="/evidence/benchmarks/">The recorded benchmarks and their limits</Link>
      </MonoCredit>
    </MonoSection>
  );
}

/** How to use Ferrugo, and what it leaves out. */
const FIT_TERMS = [
  {
    term: "Command line",
    description: (
      <>
        Install the <code>ferrugo</code> binary with <code>cargo install ferrugo --locked</code> and
        call it from scripts and jobs.
      </>
    ),
  },
  {
    term: "Rust applications",
    description: (
      <>
        Add <code>ferrugo-thumbnail</code> and <code>ferrugo-native</code> for typed errors and
        budgets you set in code.
      </>
    ),
  },
  {
    term: "Node.js, later",
    description:
      "A Node-API binding with async render and inspect calls lives in the repository. It is not published on npm yet.",
  },
  {
    term: "Not included",
    description:
      "Showing, editing or signing documents, running their scripts, or isolating the process. Ferrugo draws a page into an image.",
  },
];

export function Fit() {
  return (
    <MonoSection id="fit" title="One job, and a narrow one.">
      <p>
        Ferrugo is {lowerFirst(ferrugo.what)}. It {lowerFirst(ferrugo.does)} It turns a page into an
        image and leaves storing, showing and isolating to your application.
      </p>
      <MonoWide>
        <MonoTerms prose items={FIT_TERMS} />
      </MonoWide>
      <MonoCredit>
        <Link to="/guide/getting-started/">Install and render a first page</Link>
      </MonoCredit>
    </MonoSection>
  );
}
