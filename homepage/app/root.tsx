import { type ReactNode, useState } from "ardo/react";
import { ArdoActiveLink as NavLink, useArdoPageData, useArdoPath } from "ardo/runtime";
import {
  ArdoErrorBoundary,
  ArdoHeaderSearch,
  ArdoMobileNavigation,
  ArdoRoot,
  ArdoRootLayout,
  ArdoTOC,
} from "ardo/ui";
import { MarkDefs } from "ferramenta-family";
import touchIcon from "ferramenta-family/ferra-assets/ferrugo/apple-touch-icon.png?url";
import favicon from "ferramenta-family/ferra-assets/ferrugo/favicon.ico?url";
import {
  MonoFooter,
  MonoGuideNavigation,
  MonoHeader,
  MonoPage,
  MonoReadingLayout,
  MonoReadingTable,
  MonoTabs,
} from "ferramenta-family/monografie";
import "ferramenta-family/monografie.css";
import "ferramenta-family/monografie-docs.css";
import "ferramenta-family/monografie-ardo.css";

import { withBase } from "./components/home-data";
import navigation from "./data/navigation.json";
import site from "./data/site.json";

export function Layout({ children }: { children: ReactNode }) {
  return (
    <ArdoRootLayout
      colorScheme="light"
      className="mono-ardo"
      icon={favicon}
      appleTouchIcon={touchIcon}
      iconBasePath={false}
    >
      {children}
    </ArdoRootLayout>
  );
}
export const ErrorBoundary = ArdoErrorBoundary;

function GuideNavigation({ onNavigate }: { onNavigate?: () => void }) {
  const path = useArdoPath();
  const first = navigation[0];
  if (!first) throw new Error("Documentation navigation is empty.");
  const current =
    navigation.find((section) => path === `/${section.id}` || path.startsWith(`/${section.id}/`)) ??
    first;
  return (
    <MonoGuideNavigation aria-label="Documentation pages">
      <NavLink to="/" onClick={onNavigate}>
        Overview
      </NavLink>
      <MonoTabs aria-label="Documentation sections">
        {navigation.map((section) => (
          <NavLink
            key={section.id}
            to={section.to}
            data-active={current.id === section.id}
            onClick={onNavigate}
          >
            {section.label}
          </NavLink>
        ))}
      </MonoTabs>
      <p>{current.label}</p>
      {current.pages.map(({ title, to: href }) => (
        <NavLink key={href} to={href} onClick={onNavigate}>
          {title}
        </NavLink>
      ))}
    </MonoGuideNavigation>
  );
}

/** The documentation menu and search, which stay in the header on documentation pages. */
function DocsActions() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <ArdoMobileNavigation title="Ferrugo documentation" open={open} onOpenChange={setOpen}>
        <GuideNavigation
          onNavigate={() => {
            setOpen(false);
          }}
        />
      </ArdoMobileNavigation>
      <ArdoHeaderSearch
        variant="icon"
        onActivate={() => {
          setOpen(false);
        }}
      />
    </>
  );
}

function Header({ docs }: { docs: boolean }) {
  return (
    <MonoHeader
      product="ferrugo"
      home={withBase("/")}
      release={site.version}
      links={[
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
        <a key="github" href="https://github.com/sebastian-software/ferrugo">
          GitHub
        </a>,
      ]}
      actions={docs ? <DocsActions /> : <ArdoHeaderSearch variant="icon" />}
    />
  );
}

function Footer() {
  return (
    <MonoFooter product="ferrugo" home={withBase("/")}>
      <a href="https://github.com/sebastian-software/ferrugo">GitHub</a>
      <a href="https://crates.io/crates/ferrugo">crates.io</a>
      <a href="https://docs.rs/ferrugo-thumbnail">docs.rs</a>
    </MonoFooter>
  );
}

function Article({ children }: { children: ReactNode }) {
  const page = useArdoPageData();
  const path = useArdoPath();
  const current = navigation.find(
    (section) => path === `/${section.id}` || path.startsWith(`/${section.id}/`),
  );
  return (
    <MonoReadingLayout
      id="ferrugo-article"
      navigation={<GuideNavigation />}
      outline={<ArdoTOC source="dom" scope="#ferrugo-article" />}
    >
      <header>
        <p>Ferrugo / {current?.label ?? "Documentation"}</p>
        <h1>{page?.title.replaceAll("`", "") ?? "Documentation"}</h1>
      </header>
      {children}
    </MonoReadingLayout>
  );
}

const components = { table: MonoReadingTable, wrapper: Article };

/**
 * Every page wears the Monografie header and footer. The homepage reads on the
 * Monografie page ground; documentation on Monografie's reading layout.
 */
export default function Root() {
  const path = useArdoPath();
  const home = path === "/" || path === "";
  const shell = (
    <>
      <MarkDefs />
      <ArdoRoot
        className="mono-ardo-shell"
        scrollMode="document"
        header={<Header docs={!home} />}
        sidebar={false}
        footer={false}
        components={components}
      />
      <Footer />
    </>
  );
  return home ? (
    <MonoPage className="fg-home">{shell}</MonoPage>
  ) : (
    <MonoPage className="mono-docs fg-docs">{shell}</MonoPage>
  );
}
