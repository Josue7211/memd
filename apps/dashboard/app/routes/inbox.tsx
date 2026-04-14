import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useInbox, useInboxDismiss, usePromote, useRepair } from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import { KindBadge, StageBadge, StatusDot, ScopeLabel } from "../components/ui/badge";
import { EmptyState } from "../components/ui/empty-state";
import type { InboxMemoryItem } from "../lib/types";

export const Route = createFileRoute("/inbox")({
  component: InboxPage,
});

function InboxPage() {
  const { data, isLoading } = useInbox();
  const dismiss = useInboxDismiss();
  const promote = usePromote();
  const repair = useRepair();

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editContent, setEditContent] = useState("");

  const items = data?.items ?? [];
  const allSelected = items.length > 0 && selected.size === items.length;

  function toggleSelect(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function toggleAll() {
    setSelected(allSelected ? new Set() : new Set(items.map((i) => i.item.id)));
  }

  function handleDismiss() {
    if (selected.size === 0) return;
    dismiss.mutate({ ids: [...selected] }, {
      onSuccess: () => setSelected(new Set()),
    });
  }

  function handlePromote() {
    if (selected.size === 0) return;
    promote.mutate({ ids: [...selected] }, {
      onSuccess: () => setSelected(new Set()),
    });
  }

  function startRepair(item: InboxMemoryItem) {
    setEditingId(item.item.id);
    setEditContent(item.item.content);
  }

  function submitRepair() {
    if (!editingId) return;
    repair.mutate({ id: editingId, content: editContent }, {
      onSuccess: () => { setEditingId(null); setEditContent(""); },
    });
  }

  return (
    <div className="p-8 max-w-5xl space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-semibold tracking-tight">Inbox</h1>
          {items.length > 0 && (
            <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-status-candidate/20 text-status-candidate tabular-nums">
              {items.length}
            </span>
          )}
        </div>

        {/* Bulk actions */}
        {selected.size > 0 && (
          <div className="flex items-center gap-2">
            <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
              {selected.size} selected
            </span>
            <button
              onClick={handlePromote}
              disabled={promote.isPending}
              className="px-3 py-1.5 rounded-lg text-xs font-medium bg-status-current/15 text-status-current border border-status-current/30 hover:opacity-80 transition-opacity disabled:opacity-50"
            >
              Promote
            </button>
            <button
              onClick={handleDismiss}
              disabled={dismiss.isPending}
              className="px-3 py-1.5 rounded-lg text-xs font-medium bg-status-expired/15 text-status-expired border border-status-expired/30 hover:opacity-80 transition-opacity disabled:opacity-50"
            >
              Dismiss
            </button>
          </div>
        )}
      </div>

      {isLoading && (
        <div className="py-16 text-center text-text-tertiary text-sm">Loading...</div>
      )}

      {!isLoading && items.length === 0 && (
        <GlassPanel>
          <EmptyState title="Inbox empty" description="No items need review" />
        </GlassPanel>
      )}

      {items.length > 0 && (
        <>
          <button
            onClick={toggleAll}
            className="text-[11px] tracking-wide uppercase text-accent-bright hover:opacity-80 transition-opacity"
          >
            {allSelected ? "Deselect all" : "Select all"}
          </button>

          <div className="space-y-3">
            {items.map((entry) => (
              <InboxCard
                key={entry.item.id}
                entry={entry}
                isSelected={selected.has(entry.item.id)}
                onToggle={() => toggleSelect(entry.item.id)}
                onRepair={() => startRepair(entry)}
                isEditing={editingId === entry.item.id}
                editContent={editContent}
                onEditChange={setEditContent}
                onEditSubmit={submitRepair}
                onEditCancel={() => setEditingId(null)}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
}

function InboxCard({
  entry,
  isSelected,
  onToggle,
  onRepair,
  isEditing,
  editContent,
  onEditChange,
  onEditSubmit,
  onEditCancel,
}: {
  entry: InboxMemoryItem;
  isSelected: boolean;
  onToggle: () => void;
  onRepair: () => void;
  isEditing: boolean;
  editContent: string;
  onEditChange: (v: string) => void;
  onEditSubmit: () => void;
  onEditCancel: () => void;
}) {
  const { item, reasons } = entry;

  return (
    <GlassPanel className={isSelected ? "!border-border-active !shadow-[0_0_20px_var(--color-accent-glow)]" : ""}>
      <div className="flex items-start gap-3">
        {/* Checkbox */}
        <button
          onClick={onToggle}
          className={`mt-0.5 w-4 h-4 rounded border flex items-center justify-center shrink-0 transition-colors ${
            isSelected
              ? "bg-accent-primary border-accent-primary"
              : "border-border-subtle hover:border-border-active"
          }`}
        >
          {isSelected && (
            <svg className="w-2.5 h-2.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={3}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
            </svg>
          )}
        </button>

        <div className="flex-1 min-w-0 space-y-2">
          {/* Header */}
          <div className="flex items-center gap-2 flex-wrap">
            <KindBadge kind={item.kind} />
            <StageBadge stage={item.stage} />
            <StatusDot status={item.status} />
            <ScopeLabel value={item.scope} />
            <span className="ml-auto font-mono text-[11px] text-text-tertiary">
              {item.id.slice(0, 8)}
            </span>
          </div>

          {/* Content */}
          {isEditing ? (
            <div className="space-y-2">
              <textarea
                value={editContent}
                onChange={(e) => onEditChange(e.target.value)}
                className="w-full px-3 py-2 rounded-lg bg-bg-primary border border-border-subtle text-sm text-text-primary resize-none focus:outline-none focus:border-accent-primary transition-colors"
                rows={4}
              />
              <div className="flex gap-2">
                <button
                  onClick={onEditSubmit}
                  className="px-3 py-1 rounded-lg text-xs font-medium bg-earth-gray text-text-secondary hover:opacity-80 transition-opacity"
                >
                  Save Repair
                </button>
                <button
                  onClick={onEditCancel}
                  className="px-3 py-1 rounded-lg text-xs text-text-tertiary hover:text-text-secondary transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <p className="text-sm text-text-primary leading-relaxed">
              {item.content}
            </p>
          )}

          {/* Reasons */}
          {reasons.length > 0 && (
            <div className="flex flex-wrap gap-1.5">
              {reasons.map((reason, i) => (
                <span
                  key={i}
                  className="px-2 py-0.5 rounded text-[11px] bg-status-stale/10 text-status-stale border border-status-stale/20"
                >
                  {reason}
                </span>
              ))}
            </div>
          )}

          {/* Metadata row */}
          <div className="flex items-center gap-4 text-[11px] text-text-tertiary">
            <span className="font-mono tabular-nums">cf {Math.round(item.confidence * 100)}%</span>
            {item.project && (
              <span className="tracking-wide uppercase">project: {item.project}</span>
            )}
            {item.source_agent && (
              <span className="font-mono">{item.source_agent}</span>
            )}
            {!isEditing && (
              <button
                onClick={onRepair}
                className="ml-auto text-accent-bright hover:opacity-80 transition-opacity tracking-wide uppercase"
              >
                repair
              </button>
            )}
          </div>
        </div>
      </div>
    </GlassPanel>
  );
}
