import { defineArdoConfig } from "ardo/config";

export default defineArdoConfig({
  title: "Ferrugo",
  description:
    "Bounded previews of untrusted PDF files in Rust. Ferrugo renders one page into a small image under explicit size, time and memory limits, and names the reason when it cannot.",
  // GitHub Pages serves the repository's site under its name; there is no custom domain.
  base: "/ferrugo/",
  siteUrl: "https://sebastian-software.github.io",
  icons: false,
  markdown: { headingAttributes: true },
  linkCheck: { level: "error" },
});
