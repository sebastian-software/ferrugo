import type { ReactNode } from "ardo/react";

import { ArdoActiveLink as NavLink } from "ardo/runtime";
import {
  MonoArdoErrorBoundary,
  MonoArdoLayout,
  MonoArdoSite,
  type MonoDocsSection,
} from "ferramenta-family/ardo";
import touchIcon from "ferramenta-family/ferra-assets/ferrugo/apple-touch-icon.png?url";
import favicon from "ferramenta-family/ferra-assets/ferrugo/favicon.ico?url";

import { withBase } from "./components/home-data";
import navigation from "./data/navigation.json";
import site from "./data/site.json";
import "ferramenta-family/monografie.css";
import "ferramenta-family/monografie-docs.css";
import "ferramenta-family/monografie-ardo.css";

export function Layout({ children }: { children: ReactNode }) {
  return (
    <MonoArdoLayout icon={favicon} appleTouchIcon={touchIcon}>
      {children}
    </MonoArdoLayout>
  );
}
export const ErrorBoundary = MonoArdoErrorBoundary;

const sections: MonoDocsSection[] = navigation.map((section) => ({
  id: section.id,
  label: section.label,
  to: section.to,
  pages: section.pages.map(({ title, to }) => ({ label: title, to })),
}));

const links = [
  <NavLink key="guide" to="/guide/getting-started">
    Guide
  </NavLink>,
  <NavLink key="reference" to="/reference/errors">
    Reference
  </NavLink>,
  <NavLink key="evidence" to="/evidence/benchmarks">
    Evidence
  </NavLink>,
  <a key="api" href="https://docs.rs/ferrugo-thumbnail">
    API
  </a>,
];

/** Every page on Monografie's Ardo shell; the documentation adds its menu, search and reading layout. */
export default function Root() {
  return (
    <MonoArdoSite
      product="ferrugo"
      name="Ferrugo"
      release={site.version}
      links={links}
      repository="https://github.com/sebastian-software/ferrugo"
      sections={sections}
      home={withBase("/")}
      homeClassName="fg-home"
      docsClassName="fg-docs"
      footer={
        <>
          <a href="https://github.com/sebastian-software/ferrugo">GitHub</a>
          <a href="https://crates.io/crates/ferrugo">crates.io</a>
          <a href="https://docs.rs/ferrugo-thumbnail">docs.rs</a>
        </>
      }
    />
  );
}
