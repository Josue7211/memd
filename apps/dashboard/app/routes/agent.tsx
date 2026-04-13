import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/agent")({
  component: () => (
    <div className="p-8">
      <h1 className="text-2xl font-semibold">Agent Panel</h1>
      <p className="mt-4 text-text-secondary">Coming soon.</p>
    </div>
  ),
});
