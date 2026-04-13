import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/memory")({
  component: () => (
    <div className="p-8">
      <h1 className="text-2xl font-semibold">Memory Browser</h1>
      <p className="mt-4 text-text-secondary">Coming soon.</p>
    </div>
  ),
});
