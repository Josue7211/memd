export const siteCopy = {
  brand: "memd",
  nav: [
    { label: "Why Cloud", href: "#why-it-matters" },
    { label: "Pricing", href: "#pricing" },
    { label: "Docs", href: "/docs" },
    { label: "GitHub", href: "https://github.com/Josue7211/memd" }
  ],
  hero: {
    eyebrow: "Open source core. Paid hosted cloud.",
    title: "memd Cloud",
    description:
      "We host memd for you. No Docker. No setup. Just synced, source-linked memory that follows the work.",
    primaryCta: { label: "Start checkout", href: "/beta#checkout" },
    secondaryCta: { label: "Join waitlist", href: "/beta#waitlist" },
    bullets: [
      "OSS core stays free",
      "Cloud is the paid product",
      "Source-linked recall with provenance"
    ],
    badge: "managed hosting / no docker / open source core",
    cards: [
      {
        label: "hosted",
        title: "We run memd for you",
        body: "No self-hosting, no maintenance, no ops overhead.",
        featured: true
      },
      {
        label: "free",
        title: "OSS stays free",
        body: "Use local memd yourself if you want the self-hosted path."
      },
      {
        label: "team",
        title: "Shared memory for teams",
        body: "One workspace, multiple people, same source-linked memory."
      }
    ]
  },
  highlights: [
    {
      title: "No setup.",
      body: "We host it. You pay monthly or yearly. It just works."
    },
    {
      title: "Pay monthly.",
      body: "Subscribe for hosted memory, backup, and sync across machines."
    },
    {
      title: "Or lock in a founding plan.",
      body: "Annual upfront for early believers who want the best deal."
    }
  ],
  pricing: {
    eyebrow: "Pricing",
    title: "OSS free. Cloud paid. Founding plans for the first believers.",
    description:
      "Choose the free local path, the managed cloud, or the annual founding plan if you want the best early pricing.",
    items: [
      {
        name: "Free",
        price: "$0",
        summary: "Local OSS core for a single machine.",
        features: ["Local storage", "Basic capture and resume", "Community support"],
        cta: { label: "Get OSS", href: "/docs" }
      },
      {
        name: "Cloud",
        price: "$29/mo",
        summary: "We host memd for you.",
        features: ["Hosted sync", "Source-linked recall", "Backup and restore"],
        cta: { label: "Buy cloud", href: "/beta#checkout" },
        featured: true
      },
      {
        name: "Founding",
        price: "$299/yr",
        summary: "Best early rate, billed upfront.",
        features: ["Priority onboarding", "Founder pricing", "Early access"],
        cta: { label: "Lock founding plan", href: "/beta#checkout" }
      },
      {
        name: "Team",
        price: "$49/seat",
        summary: "Shared memory for small teams.",
        features: ["Shared workspaces", "Team policies", "Audit history"],
        cta: { label: "Join team waitlist", href: "/beta#waitlist" }
      }
    ]
  },
  cta: {
    eyebrow: "Ready to start",
    title: "Want memd Cloud without setup pain?",
    description:
      "Start on the free OSS path or jump straight into the hosted paid beta. We’ll handle the running parts.",
    primaryCta: { label: "Start checkout", href: "/beta#checkout" },
    secondaryCta: { label: "Join waitlist", href: "/beta#waitlist" }
  },
  footer: [
    { label: "Docs", href: "/docs" },
    { label: "GitHub", href: "https://github.com/Josue7211/memd" },
    { label: "License", href: "https://www.gnu.org/licenses/agpl-3.0.en.html" }
  ]
} as const;
