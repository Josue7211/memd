import { createRootRoute, Link, Outlet } from "@tanstack/react-router";

const nav = [
  { to: "/", label: "Status" },
  { to: "/memory", label: "Memory" },
  { to: "/inbox", label: "Inbox" },
  { to: "/graph", label: "Graph" },
  { to: "/atlas", label: "Atlas" },
  { to: "/procedures", label: "Procedures" },
  { to: "/agent", label: "Agent" },
  { to: "/ask", label: "Ask" },
] as const;

export const Route = createRootRoute({
  component: Shell,
  notFoundComponent: () => (
    <div className="flex items-center justify-center h-screen text-text-secondary">
      404 — not found
    </div>
  ),
});

function Shell() {
  return (
    <div className="flex min-h-screen bg-bg-primary text-text-primary font-sans">
      <nav className="w-52 shrink-0 border-r border-border-subtle bg-bg-surface/60 backdrop-blur-xl flex flex-col">
        <div className="px-5 py-4 border-b border-border-subtle">
          <span className="text-lg font-semibold tracking-tight text-accent-primary">
            memd
          </span>
        </div>

        <div className="flex-1 py-3 flex flex-col gap-0.5">
          {nav.map((item) => (
            <Link
              key={item.to}
              to={item.to}
              activeOptions={{ exact: item.to === "/" }}
              className="mx-2 px-3 py-2 rounded-lg text-sm text-text-secondary hover:text-text-primary hover:bg-white/[0.04] transition-colors"
              activeProps={{
                className:
                  "mx-2 px-3 py-2 rounded-lg text-sm text-accent-bright bg-accent-primary/10 border border-border-active",
              }}
            >
              {item.label}
            </Link>
          ))}
        </div>

        <div className="px-5 py-3 border-t border-border-subtle text-xs text-text-tertiary">
          memd dashboard
        </div>
      </nav>

      <main className="flex-1 min-w-0 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
