import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useSearch } from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import { KindBadge, StageBadge, StatusDot, ConfidenceBar, ScopeLabel } from "../components/ui/badge";
import { EmptyState } from "../components/ui/empty-state";
import type {
  MemoryKind,
  MemoryScope,
  MemoryStatus,
  MemoryStage,
  MemoryItem,
  SearchMemoryRequest,
} from "../lib/types";

export const Route = createFileRoute("/memory")({
  component: MemoryBrowser,
});

const ALL_KINDS: MemoryKind[] = [
  "fact", "decision", "preference", "runbook", "procedural",
  "self_model", "topology", "status", "live_truth", "pattern", "constraint",
];
const ALL_SCOPES: MemoryScope[] = ["local", "synced", "project", "global"];
const ALL_STATUSES: MemoryStatus[] = ["active", "stale", "superseded", "contested", "expired"];
const ALL_STAGES: MemoryStage[] = ["canonical", "candidate"];

function MemoryBrowser() {
  const [query, setQuery] = useState("");
  const [scopes, setScopes] = useState<MemoryScope[]>([...ALL_SCOPES]);
  const [kinds, setKinds] = useState<MemoryKind[]>([...ALL_KINDS]);
  const [statuses, setStatuses] = useState<MemoryStatus[]>(["active"]);
  const [stages, setStages] = useState<MemoryStage[]>([...ALL_STAGES]);
  const [limit, setLimit] = useState(20);
  const [expanded, setExpanded] = useState<string | null>(null);

  const searchReq: SearchMemoryRequest = {
    query: query || undefined,
    scopes,
    kinds,
    statuses,
    stages,
    tags: [],
    limit,
  };

  const canSearch = scopes.length > 0 && kinds.length > 0 && statuses.length > 0 && stages.length > 0;
  const { data, isLoading, error } = useSearch(searchReq, canSearch);

  return (
    <div className="p-8 max-w-6xl space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">Memory Browser</h1>

      {/* Search bar */}
      <div className="flex gap-3">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search memories..."
          className="flex-1 px-4 py-2.5 rounded-lg bg-bg-primary border border-border-subtle text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary transition-colors"
        />
        <select
          value={limit}
          onChange={(e) => setLimit(Number(e.target.value))}
          className="px-3 py-2 rounded-lg bg-bg-primary border border-border-subtle text-sm text-text-secondary focus:outline-none focus:border-accent-primary"
        >
          <option value={10}>10</option>
          <option value={20}>20</option>
          <option value={50}>50</option>
          <option value={100}>100</option>
        </select>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap gap-4">
        <FilterGroup label="Scope" options={ALL_SCOPES} selected={scopes} onChange={setScopes} />
        <FilterGroup label="Status" options={ALL_STATUSES} selected={statuses} onChange={setStatuses} />
        <FilterGroup label="Stage" options={ALL_STAGES} selected={stages} onChange={setStages} />
        <FilterGroup label="Kind" options={ALL_KINDS} selected={kinds} onChange={setKinds} />
      </div>

      {/* Results */}
      <GlassPanel padding="none">
        {isLoading && (
          <div className="p-8 text-center text-text-tertiary text-sm">Loading...</div>
        )}
        {error && (
          <div className="p-8 text-center text-status-expired text-sm">
            {(error as Error).message}
          </div>
        )}
        {data && data.items.length === 0 && (
          <EmptyState title="No results" description="Adjust filters or search query" />
        )}
        {data && data.items.length > 0 && (
          <div>
            {data.items.map((item) => (
              <MemoryRow
                key={item.id}
                item={item}
                isExpanded={expanded === item.id}
                onToggle={() => setExpanded(expanded === item.id ? null : item.id)}
              />
            ))}
          </div>
        )}
      </GlassPanel>

      {data && (
        <p className="text-[11px] tracking-wide uppercase text-text-tertiary">
          Showing {data.items.length} items · route: {data.route} · intent: {data.intent}
        </p>
      )}
    </div>
  );
}

function MemoryRow({
  item,
  isExpanded,
  onToggle,
}: {
  item: MemoryItem;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  return (
    <div className="group">
      <button
        onClick={onToggle}
        className="w-full flex items-center gap-3 px-5 py-2.5 text-left hover:bg-hover transition-colors border-b border-border-subtle last:border-0"
      >
        <KindBadge kind={item.kind} />
        <StageBadge stage={item.stage} />
        <span className="flex-1 truncate text-sm text-text-primary">
          {item.content.slice(0, 140)}
        </span>
        <StatusDot status={item.status} />
        <ScopeLabel value={item.scope} />
      </button>

      {isExpanded && (
        <div className="px-5 pb-5 space-y-4 border-b border-border-subtle bg-hover">
          {/* Content */}
          <div className="pt-3">
            <p className="text-sm text-text-primary whitespace-pre-wrap leading-relaxed">
              {item.content}
            </p>
          </div>

          {/* Metadata grid */}
          <div className="grid grid-cols-3 gap-3 text-xs">
            <MetaField label="ID" value={item.id} mono />
            <MetaField label="Project" value={item.project ?? "—"} />
            <MetaField label="Namespace" value={item.namespace ?? "—"} />
            <MetaField label="Source Agent" value={item.source_agent ?? "—"} mono />
            <MetaField label="Source System" value={item.source_system ?? "—"} />
            <MetaField label="Source Quality" value={item.source_quality ?? "—"} />
            <MetaField label="Created" value={formatDate(item.created_at)} />
            <MetaField label="Updated" value={formatDate(item.updated_at)} />
            <MetaField label="Verified" value={item.last_verified_at ? formatDate(item.last_verified_at) : "never"} />
          </div>

          {/* Confidence */}
          <div className="max-w-xs">
            <ConfidenceBar value={item.confidence} />
          </div>

          {/* Tags */}
          {item.tags.length > 0 && (
            <div className="flex flex-wrap gap-1.5">
              {item.tags.map((tag) => (
                <span
                  key={tag}
                  className="px-2 py-0.5 rounded text-[11px] bg-white/5 text-text-tertiary border border-white/5"
                >
                  {tag}
                </span>
              ))}
            </div>
          )}

          {/* Supersedes */}
          {item.supersedes.length > 0 && (
            <div className="text-xs text-text-tertiary">
              <span className="tracking-wide uppercase">Supersedes: </span>
              <span className="font-mono">{item.supersedes.map((id) => id.slice(0, 8)).join(", ")}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function MetaField({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-0.5">{label}</p>
      <p className={`text-text-secondary truncate ${mono ? "font-mono text-[11px]" : "text-xs"}`}>
        {value}
      </p>
    </div>
  );
}

function FilterGroup<T extends string>({
  label,
  options,
  selected,
  onChange,
}: {
  label: string;
  options: T[];
  selected: T[];
  onChange: (v: T[]) => void;
}) {
  const allSelected = selected.length === options.length;

  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2">
        <span className="text-[11px] tracking-wide uppercase text-text-tertiary">{label}</span>
        <button
          onClick={() => onChange(allSelected ? [] : [...options])}
          className="text-[10px] text-accent-bright hover:opacity-80 transition-opacity"
        >
          {allSelected ? "none" : "all"}
        </button>
      </div>
      <div className="flex flex-wrap gap-1">
        {options.map((opt) => {
          const active = selected.includes(opt);
          return (
            <button
              key={opt}
              onClick={() =>
                onChange(
                  active
                    ? selected.filter((s) => s !== opt)
                    : [...selected, opt],
                )
              }
              className={`px-2 py-0.5 rounded text-[11px] border transition-colors ${
                active
                  ? "bg-accent-primary/15 text-accent-bright border-accent-primary/40"
                  : "bg-glass text-text-tertiary border-border-subtle hover:border-border-active"
              }`}
            >
              {opt.replace(/_/g, " ")}
            </button>
          );
        })}
      </div>
    </div>
  );
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  const diff = Date.now() - d.getTime();

  if (diff < 60_000) return "just now";
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}
