import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import {
  useProcedures,
  useProcedurePromote,
  useProcedureRetire,
} from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import {
  ProcedureStatusBadge,
  ConfidenceBar,
} from "../components/ui/badge";
import { EmptyState } from "../components/ui/empty-state";
import type { Procedure, ProcedureStatus, ProcedureKind } from "../lib/types";

export const Route = createFileRoute("/procedures")({
  component: ProceduresPage,
});

const TABS: { label: string; status: ProcedureStatus }[] = [
  { label: "Promoted", status: "promoted" },
  { label: "Candidate", status: "candidate" },
  { label: "Retired", status: "retired" },
];

const kindMap: Record<ProcedureKind, string> = {
  workflow: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
  policy: "bg-sky-500/15 text-sky-300 border-sky-500/30",
  recovery: "bg-amber-500/15 text-amber-300 border-amber-500/30",
};

function ProceduresPage() {
  const [tab, setTab] = useState<ProcedureStatus>("promoted");
  const [expanded, setExpanded] = useState<string | null>(null);
  const { data, isLoading } = useProcedures({ status: tab });
  const promote = useProcedurePromote();
  const retire = useProcedureRetire();

  return (
    <div className="p-8 max-w-6xl space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">Procedures</h1>

      {/* Tabs */}
      <div className="flex gap-1">
        {TABS.map((t) => (
          <button
            key={t.status}
            onClick={() => {
              setTab(t.status);
              setExpanded(null);
            }}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              tab === t.status
                ? "bg-accent-primary/15 text-accent-bright border border-accent-primary/40"
                : "text-text-tertiary hover:text-text-secondary border border-transparent"
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* List */}
      <GlassPanel padding="none">
        {isLoading && (
          <div className="p-8 text-center text-text-tertiary text-sm">
            Loading...
          </div>
        )}
        {data && data.procedures.length === 0 && (
          <EmptyState title={`No ${tab} procedures`} />
        )}
        {data?.procedures.map((proc) => (
          <ProcedureRow
            key={proc.id}
            procedure={proc}
            isExpanded={expanded === proc.id}
            onToggle={() =>
              setExpanded(expanded === proc.id ? null : proc.id)
            }
            onPromote={() => promote.mutate({ id: proc.id })}
            onRetire={() => retire.mutate({ id: proc.id })}
          />
        ))}
      </GlassPanel>
    </div>
  );
}

function ProcedureRow({
  procedure: p,
  isExpanded,
  onToggle,
  onPromote,
  onRetire,
}: {
  procedure: Procedure;
  isExpanded: boolean;
  onToggle: () => void;
  onPromote: () => void;
  onRetire: () => void;
}) {
  return (
    <div className="group">
      <button
        onClick={onToggle}
        className="w-full flex items-center gap-3 px-5 py-3 text-left hover:bg-hover transition-colors border-b border-border-subtle last:border-0"
      >
        <span
          className={`inline-flex items-center px-2 py-0.5 rounded text-[11px] font-medium border ${kindMap[p.kind]}`}
        >
          {p.kind}
        </span>
        <ProcedureStatusBadge status={p.status} />
        <span className="flex-1 truncate text-sm text-text-primary">
          {p.name}
        </span>
        <span className="text-xs text-text-tertiary tabular-nums">
          {p.use_count} uses
        </span>
        <div className="w-20">
          <ConfidenceBar value={p.confidence} />
        </div>
      </button>

      {isExpanded && (
        <div className="px-5 pb-5 space-y-4 border-b border-border-subtle bg-hover">
          {p.description && (
            <p className="text-sm text-text-secondary pt-3">
              {p.description}
            </p>
          )}

          {p.trigger && (
            <div className="text-xs">
              <span className="text-text-tertiary uppercase tracking-wide">
                Trigger:{" "}
              </span>
              <span className="text-text-secondary">{p.trigger}</span>
            </div>
          )}

          {/* Steps */}
          {p.steps.length > 0 && (
            <div>
              <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-2">
                Steps
              </p>
              <ol className="list-decimal list-inside space-y-1 text-sm text-text-secondary">
                {p.steps.map((step, i) => (
                  <li key={i}>{step}</li>
                ))}
              </ol>
            </div>
          )}

          {p.success_criteria && (
            <div className="text-xs">
              <span className="text-text-tertiary uppercase tracking-wide">
                Success:{" "}
              </span>
              <span className="text-text-secondary">
                {p.success_criteria}
              </span>
            </div>
          )}

          {p.tags.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {p.tags.map((t) => (
                <span
                  key={t}
                  className="px-2 py-0.5 rounded text-[11px] bg-white/5 text-text-tertiary border border-white/5"
                >
                  {t}
                </span>
              ))}
            </div>
          )}

          {/* Source traces */}
          {p.source_ids.length > 0 && (
            <div>
              <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-1">
                Sources
              </p>
              <div className="flex flex-wrap gap-1.5">
                {p.source_ids.map((id) => (
                  <span
                    key={id}
                    className="font-mono text-[11px] text-accent-bright"
                  >
                    {id.slice(0, 12)}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2 pt-2">
            {p.status === "candidate" && (
              <button
                onClick={onPromote}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 hover:bg-emerald-500/25 transition-colors"
              >
                Promote
              </button>
            )}
            {p.status === "promoted" && (
              <button
                onClick={onRetire}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-red-500/15 text-red-300 border border-red-500/30 hover:bg-red-500/25 transition-colors"
              >
                Retire
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
