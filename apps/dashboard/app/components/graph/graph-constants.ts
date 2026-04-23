/** Force config from supermemory E2-D3 donor extraction */
export const FORCE_CONFIG = {
  charge: -2000,
  alphaDecay: 0.025,
  warmupTicks: 150,
} as const;

/** Node radius by type */
export function nodeRadius(node: {
  type?: string;
  nodeCount?: number;
  confidence?: number;
}) {
  if (node.type === "region")
    return Math.max(12, Math.min(40, (node.nodeCount ?? 1) * 2));
  return 4 + (node.confidence ?? 0.5) * 8;
}

/** Kind → color mapping (matches badge.tsx kindColors) */
export const KIND_COLORS: Record<string, string> = {
  fact: "#a855f7",
  decision: "#8b5cf6",
  preference: "#6366f1",
  runbook: "#0ea5e9",
  procedural: "#10b981",
  self_model: "#f59e0b",
  topology: "#06b6d4",
  status: "#71717a",
  live_truth: "#f43f5e",
  pattern: "#d946ef",
  constraint: "#ef4444",
  region: "#8b5cf6",
};

/** Relation → color mapping */
export const RELATION_COLORS: Record<string, string> = {
  same_as: "#8b5cf6",
  derived_from: "#06b6d4",
  supersedes: "#f59e0b",
  contradicts: "#ef4444",
  related: "#555570",
};
