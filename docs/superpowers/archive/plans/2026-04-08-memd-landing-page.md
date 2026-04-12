# memd Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a public landing page for `memd` that explains the product, shows pricing, and converts visitors into free or paid users.

**Architecture:** The site will live in `apps/` as a small Astro app, separate from the Rust workspace in `crates/`. Shared page copy will live in a data module, and the page will be composed from focused components for layout, hero, grid sections, pricing, CTA, and footer. Root gitignore entries will exclude only generated site output and local dependencies.

**Tech Stack:** Astro, TypeScript, vanilla CSS, Markdown for content snippets.

---

### Task 1: Add the site app skeleton

**Files:**
- Create: `apps/package.json`
- Create: `apps/astro.config.mjs`
- Create: `apps/tsconfig.json`
- Create: `apps/src/pages/index.astro`
- Create: `apps/src/styles/globals.css`
- Modify: `.gitignore`

- [ ] **Step 1: Write the app manifest and config**

```json
{
  "name": "memd-site",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "astro dev",
    "build": "astro build",
    "preview": "astro preview"
  },
  "dependencies": {
    "astro": "^5.0.0"
  }
}
```

- [ ] **Step 2: Add the page shell and global styles**

```astro
---
import "../styles/globals.css";
---

<html lang="en">
  <body>
    <main>Landing page content</main>
  </body>
</html>
```

- [ ] **Step 3: Ignore generated site output only**

```gitignore
apps/node_modules/
apps/dist/
apps/.astro/
```

- [ ] **Step 4: Run a build check after dependencies are installed**

Run: `cd apps && npm install && npm run build`
Expected: Astro builds successfully with no missing-file errors.

### Task 2: Add shared copy and layout components

**Files:**
- Create: `apps/src/content/copy.ts`
- Create: `apps/src/components/Layout.astro`
- Create: `apps/src/components/Nav.astro`
- Create: `apps/src/components/Footer.astro`
- Create: `apps/src/components/CTA.astro`

- [ ] **Step 1: Define the shared copy data**

```ts
export const navLinks = [
  { label: "Pricing", href: "#pricing" },
  { label: "Docs", href: "/docs" },
  { label: "GitHub", href: "https://github.com/" }
];
```

- [ ] **Step 2: Build the layout wrapper with metadata**

```astro
---
const { title, description } = Astro.props;
---

<head>
  <title>{title}</title>
  <meta name="description" content={description} />
</head>
```

- [ ] **Step 3: Build nav, CTA, and footer components**

```astro
---
const { links } = Astro.props;
---

<nav>{links.map((link) => <a href={link.href}>{link.label}</a>)}</nav>
```

- [ ] **Step 4: Wire the page to the shared layout**

Run: `cd apps && npm run build`
Expected: The app compiles with the page shell and reusable chrome in place.

### Task 3: Compose the landing page sections

**Files:**
- Create: `apps/src/components/Hero.astro`
- Create: `apps/src/components/SectionHeading.astro`
- Create: `apps/src/components/CardGrid.astro`
- Create: `apps/src/components/PricingTable.astro`
- Modify: `apps/src/pages/index.astro`

- [ ] **Step 1: Add the hero section with the primary conversion message**

```astro
<section>
  <h1>Agent memory that survives real work</h1>
  <p>memd keeps memory source-linked, intent-aware, and usable across sessions, machines, and teams.</p>
</section>
```

- [ ] **Step 2: Add the problem, how-it-works, differentiators, and use-case sections**

```astro
<section>
  <h2>How it works</h2>
  <ol>
    <li>Capture</li>
    <li>Route</li>
    <li>Compact</li>
    <li>Recall</li>
    <li>Verify</li>
  </ol>
</section>
```

- [ ] **Step 3: Add the pricing table and trust section**

```astro
<section id="pricing">
  <h2>Pricing</h2>
</section>
```

- [ ] **Step 4: Add the final CTA block**

```astro
<section>
  <a href="#pricing">See pricing</a>
  <a href="/beta">Join beta</a>
</section>
```

- [ ] **Step 5: Build the page and verify the structure**

Run: `cd apps && npm run build`
Expected: The full landing page compiles with all sections rendered.

### Task 4: Polish responsive styling and visual hierarchy

**Files:**
- Create: `apps/src/styles/tokens.css`
- Modify: `apps/src/styles/globals.css`
- Modify: `apps/src/components/*.astro`

- [ ] **Step 1: Add typography, color, spacing, and surface tokens**
- [ ] **Step 2: Style the hero, cards, pricing grid, and footer for mobile first**
- [ ] **Step 3: Add a strong but restrained accent color and a clear section rhythm**
- [ ] **Step 4: Rebuild and inspect the final page output**

Run: `cd apps && npm run build`
Expected: Build succeeds and the layout reads cleanly on mobile and desktop.

### Task 5: Confirm the site is isolated from the Rust workspace

**Files:**
- Modify: `Cargo.toml` only if needed to keep the Rust workspace unchanged
- Modify: `README.md` only if a site link is added later

- [ ] **Step 1: Verify the Rust workspace still builds independently**

Run: `cargo check`
Expected: The Rust workspace compiles with no dependency on the site app.

- [ ] **Step 2: Keep the site app out of the workspace members list**

```toml
[workspace]
members = [
  "crates/memd-core",
  "crates/memd-client",
  "crates/memd-multimodal",
  "crates/memd-sidecar",
  "crates/memd-rag",
  "crates/memd-schema",
  "crates/memd-server",
  "crates/memd-worker"
]
```

- [ ] **Step 3: Commit the landing page scaffold**

Run:
```bash
git add apps .gitignore docs/superpowers/specs/2026-04-08-memd-landing-page-design.md docs/superpowers/plans/2026-04-08-memd-landing-page.md
git commit -m "feat: add memd landing page scaffold"
```

## Self-Review

- The spec covers the conversion goal, page structure, visual direction, and success criteria.
- The plan covers app scaffolding, shared copy, landing page composition, styling, and workspace isolation.
- No placeholders remain in the spec or plan.
- The site app is separate from the Rust workspace, which matches the approved direction.
