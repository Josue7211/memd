import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useSearch, useExplain } from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import {
  KindBadge,
  StageBadge,
  StatusDot,
  ConfidenceBar,
  ScopeLabel,
} from "../components/ui/badge";
import { EmptyState } from "../components/ui/empty-state";
import type { MemoryItem, SearchMemoryRequest } from "../lib/types";

export const Route = createFileRoute("/ask")({
  component: AskPage,
});

function AskPage() {
  const [query, setQuery] = useState("");
  const [submitted, setSubmitted] = useState("");
  const [explaining, setExplaining] = useState<string | null>(null);

  const searchReq: SearchMemoryRequest = {
    query: submitted || undefined,
    scopes: ["local", "synced", "project", "global"],
    kinds: [
      "fact",
      "decision",
      "preference",
      "runbook",
      "procedural",
      "self_model",
      "topology",
      "live_truth",
      "pattern",
      "constraint",
    ],
    statuses: ["active"],
    stages: ["canonical", "candidate"],
    tags: [],
    limit: 20,
  };

  const { data, isLoading } = useSearch(searchReq, !!submitted);
  const explain = useExplain(explaining ?? "");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitted(query.trim());
    setExplaining(null);
  };

  return (
    <div className="p-8 max-w-4xl mx-auto space-y-8">
      <h1 className="text-2xl font-semibold tracking-tight text-center">
        Ask memd
      </h1>

      {/* Search bar */}
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="What do you want to know?"
          className="w-full px-6 py-4 rounded-xl bg-bg-surface border border-border-subtle text-lg text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary transition-colors"
          autoFocus
        />
      </form>

      {/* Results */}
      {isLoading && (
        <div className="text-center text-text-tertiary text-sm">
          Searching...
        </div>
      )}

      {data && data.items.length === 0 && submitted && (
        <EmptyState
          title="No results"
          description="Try different words or broaden your search"
        />
      )}

      {data && data.items.length > 0 && (
        <div className="space-y-3">
          {data.items.map((item) => (
            <AskResult
              key={item.id}
              item={item}
              isExplaining={explaining === item.id}
              explanation={
                explaining === item.id
                  ? explain.data?.explanation
                  : undefined
              }
              onExplain={() =>
                setExplaining(explaining === item.id ? null : item.id)
              }
            />
          ))}
        </div>
      )}
    </div>
  );
}

function AskResult({
  item,
  isExplaining,
  explanation,
  onExplain,
}: {
  item: MemoryItem;
  isExplaining: boolean;
  explanation?: string;
  onExplain: () => void;
}) {
  return (
    <GlassPanel hover>
      <div className="flex items-start gap-3">
        <div className="flex flex-wrap gap-1.5 shrink-0">
          <KindBadge kind={item.kind} />
          <StageBadge stage={item.stage} />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm text-text-primary leading-relaxed">
            {item.content}
          </p>
          <div className="flex items-center gap-3 mt-2">
            <StatusDot status={item.status} />
            <ScopeLabel value={item.scope} />
            <div className="w-24">
              <ConfidenceBar value={item.confidence} />
            </div>
            <button
              onClick={onExplain}
              className="ml-auto text-xs text-accent-bright hover:opacity-80 transition-opacity"
            >
              {isExplaining ? "Hide" : "Explain"}
            </button>
          </div>
        </div>
      </div>

      {isExplaining && explanation && (
        <div className="mt-3 pt-3 border-t border-border-subtle">
          <p className="text-xs text-text-secondary whitespace-pre-wrap">
            {explanation}
          </p>
        </div>
      )}
    </GlassPanel>
  );
}
