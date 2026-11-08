import { ArdoLink as Link } from "ardo/runtime";
import { MonoCredit, MonoSection, MonoTerms, MonoWide } from "ferramenta-family/monografie";

import { ferrugo, lowerFirst } from "./home-data";

/* The homepage's engine chapter: fit. */

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
