import type { GraphMode } from "./use-graph-mode";

export function GraphToggle({
  mode,
  onToggle,
}: {
  mode: GraphMode;
  onToggle: () => void;
}) {
  return (
    <button
      onClick={onToggle}
      className="px-3 py-1.5 rounded-lg text-xs font-medium border border-border-subtle bg-bg-surface/60 text-text-secondary hover:border-border-active hover:text-accent-bright transition-colors"
    >
      {mode === "2d" ? "Switch to 3D" : "Switch to 2D"}
    </button>
  );
}
