import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import {
  useSessions,
  useProfile,
  useWorking,
  useQueenDeny,
  useQueenReroute,
  useQueenHandoff,
} from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import { EmptyState } from "../components/ui/empty-state";
import type { HiveSessionRecord } from "../lib/types";

export const Route = createFileRoute("/agent")({
  component: AgentPage,
});

function AgentPage() {
  const sessions = useSessions();
  const [selected, setSelected] = useState<HiveSessionRecord | null>(null);

  const agents = sessions.data?.sessions ?? [];
  const agentName = selected?.effective_agent ?? selected?.agent ?? "";
  const profile = useProfile(agentName, selected?.project ?? undefined);
  const working = useWorking(
    selected?.project ? { project: selected.project } : undefined,
  );

  const deny = useQueenDeny();
  const reroute = useQueenReroute();
  const handoff = useQueenHandoff();

  return (
    <div className="flex h-screen">
      {/* Agent list */}
      <div className="w-64 shrink-0 border-r border-border-subtle bg-bg-surface/60 overflow-y-auto">
        <div className="px-4 py-3 border-b border-border-subtle">
          <h2 className="text-sm font-medium text-text-secondary">
            Sessions
          </h2>
        </div>
        {agents.length === 0 && <EmptyState title="No active sessions" />}
        {agents.map((s) => (
          <button
            key={s.session}
            onClick={() => setSelected(s)}
            className={`w-full text-left px-4 py-3 border-b border-border-subtle hover:bg-hover transition-colors ${
              selected?.session === s.session
                ? "bg-accent-primary/10 border-l-2 border-l-accent-primary"
                : ""
            }`}
          >
            <p className="text-sm font-mono text-accent-bright truncate">
              {s.effective_agent ?? s.agent ?? s.session.slice(0, 12)}
            </p>
            <div className="flex items-center gap-2 mt-1">
              {s.hive_role && (
                <span className="text-[10px] uppercase tracking-wide text-text-tertiary">
                  {s.hive_role}
                </span>
              )}
              <span className="text-[10px] uppercase tracking-wide text-text-tertiary">
                {s.status}
              </span>
            </div>
            {s.project && (
              <p className="text-xs text-text-tertiary mt-0.5 truncate">
                {s.project}
              </p>
            )}
          </button>
        ))}
      </div>

      {/* Detail */}
      <div className="flex-1 overflow-y-auto p-8 space-y-6">
        {!selected ? (
          <EmptyState
            title="Select an agent"
            description="Choose from the sidebar to view details"
          />
        ) : (
          <>
            <h1 className="text-2xl font-semibold tracking-tight">
              {selected.effective_agent ??
                selected.agent ??
                "Unknown Agent"}
            </h1>

            {/* Session info */}
            <GlassPanel>
              <h2 className="text-sm font-medium text-text-secondary mb-3">
                Session
              </h2>
              <div className="grid grid-cols-3 gap-3 text-xs">
                <Field label="Session ID" value={selected.session} mono />
                <Field label="Status" value={selected.status} />
                <Field
                  label="Hive Role"
                  value={selected.hive_role ?? "—"}
                />
                <Field label="Project" value={selected.project ?? "—"} />
                <Field
                  label="Namespace"
                  value={selected.namespace ?? "—"}
                />
                <Field
                  label="Worker"
                  value={selected.worker_name ?? "—"}
                />
                <Field
                  label="Last Seen"
                  value={selected.last_seen ?? "—"}
                />
                <Field
                  label="Last Wake"
                  value={selected.last_wake_at ?? "—"}
                />
                <Field label="Focus" value={selected.focus ?? "—"} />
              </div>
            </GlassPanel>

            {/* Profile */}
            {profile.data?.profile && (
              <GlassPanel>
                <h2 className="text-sm font-medium text-text-secondary mb-3">
                  Profile
                </h2>
                <div className="text-xs space-y-2">
                  {profile.data.profile.capabilities &&
                    profile.data.profile.capabilities.length > 0 && (
                      <div>
                        <span className="text-text-tertiary uppercase tracking-wide">
                          Capabilities
                        </span>
                        <div className="flex flex-wrap gap-1 mt-1">
                          {profile.data.profile.capabilities.map((c) => (
                            <span
                              key={c}
                              className="px-2 py-0.5 rounded text-[11px] bg-white/5 text-text-tertiary border border-white/5"
                            >
                              {c}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}
                </div>
              </GlassPanel>
            )}

            {/* Working memory summary */}
            {working.data && (
              <GlassPanel>
                <h2 className="text-sm font-medium text-text-secondary mb-3">
                  Working Memory
                </h2>
                <div className="text-xs text-text-tertiary">
                  {working.data.records.length} items ·{" "}
                  {working.data.used_chars}/{working.data.budget_chars} chars
                  {working.data.truncated && " (truncated)"}
                </div>
              </GlassPanel>
            )}

            {/* Hive controls */}
            <GlassPanel>
              <h2 className="text-sm font-medium text-text-secondary mb-3">
                Hive Controls
              </h2>
              <div className="flex gap-2">
                <button
                  onClick={() =>
                    reroute.mutate({ session: selected.session })
                  }
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-sky-500/15 text-sky-300 border border-sky-500/30 hover:bg-sky-500/25 transition-colors"
                >
                  Reroute
                </button>
                <button
                  onClick={() =>
                    deny.mutate({ session: selected.session })
                  }
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-amber-500/15 text-amber-300 border border-amber-500/30 hover:bg-amber-500/25 transition-colors"
                >
                  Deny
                </button>
                <button
                  onClick={() =>
                    handoff.mutate({ session: selected.session })
                  }
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 hover:bg-emerald-500/25 transition-colors"
                >
                  Handoff
                </button>
              </div>
            </GlassPanel>
          </>
        )}
      </div>
    </div>
  );
}

function Field({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div>
      <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-0.5">
        {label}
      </p>
      <p
        className={`text-text-secondary truncate ${mono ? "font-mono text-[11px]" : ""}`}
      >
        {value}
      </p>
    </div>
  );
}
