import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/graph")({
  component: () => (
    <div className="p-8">
      <h1 className="text-2xl font-semibold">Graph</h1>
      <p className="mt-4 text-text-secondary">Coming soon.</p>
    </div>
  ),
});
