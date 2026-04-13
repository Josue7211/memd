import { createFileRoute } from "@tanstack/react-router";
import { useHealth, useWorking, useInbox, useSessions } from "../lib/queries";
import { MetricCard } from "../components/ui/metric-card";
import { GlassPanel } from "../components/ui/glass-panel";
import { KindBadge, StatusDot } from "../components/ui/badge";
import { EmptyState } from "../components/ui/empty-state";

export const Route = createFileRoute("/")({
  component: StatusDashboard,
});

function StatusDashboard() {
  const health = useHealth();
  const working = useWorking();
  const inbox = useInbox();
  const sessions = useSessions();

  const h = health.data;
  const isConnected = health.isSuccess;

  return (
    <div className="p-8 max-w-6xl space-y-8">
      <div className="flex items-center gap-3">
        <h1 className="text-2xl font-semibold">Status</h1>
        <span
          className={`w-2 h-2 rounded-full ${isConnected ? "bg-status-current" : "bg-status-expired"}`}
        />
      </div>

      {/* Metrics row */}
      <div className="grid grid-cols-4 gap-4">
        <MetricCard
          label="Eval Score"
          value={h?.eval_score != null ? Math.round(h.eval_score) : "—"}
          color={
            h?.degraded
              ? "text-status-expired"
              : h?.eval_score != null
                ? "text-status-current"
                : "text-text-primary"
          }
          sub={h?.degraded ? "degraded" : undefined}
        />
        <MetricCard
          label="Inbox"
          value={inbox.data?.items.length ?? "—"}
          color="text-status-candidate"
        />
        <MetricCard
          label="Working Memory"
          value={working.data?.items.length ?? "—"}
          color="text-text-primary"
          sub={
            working.data
              ? `${working.data.items.filter((i) => i.kind !== "status").length} non-status`
              : undefined
          }
        />
        <MetricCard
          label="Sessions"
          value={sessions.data?.sessions.length ?? "—"}
          color="text-accent-bright"
        />
      </div>

      {/* Pressure metrics */}
      {h && (
        <div className="grid grid-cols-4 gap-4">
          <PressureCell label="Candidates" value={h.candidate_count} />
          <PressureCell label="Stale" value={h.stale_count} warn />
          <PressureCell label="Expired" value={h.expired_count} danger />
          <PressureCell label="Items" value={h.items} />
        </div>
      )}

      {/* Working memory summary */}
      <GlassPanel>
        <h2 className="text-sm font-medium text-text-secondary mb-4">
          Working Memory
        </h2>
        {working.data?.items.length ? (
          <div className="space-y-2">
            {working.data.items.slice(0, 8).map((item) => (
              <div
                key={item.id}
                className="flex items-center gap-3 text-sm py-1.5 border-b border-border-subtle last:border-0"
              >
                <KindBadge kind={item.kind} />
                <span className="flex-1 truncate text-text-primary">
                  {item.content.slice(0, 120)}
                </span>
                <StatusDot status={item.status} />
              </div>
            ))}
          </div>
        ) : (
          <EmptyState
            title={isConnected ? "No working memory" : "Not connected"}
            description={
              isConnected ? undefined : "Start memd serve to see data"
            }
          />
        )}
      </GlassPanel>

      {/* Active sessions */}
      <GlassPanel>
        <h2 className="text-sm font-medium text-text-secondary mb-4">
          Active Sessions
        </h2>
        {sessions.data?.sessions.length ? (
          <div className="space-y-2">
            {sessions.data.sessions.map((s) => (
              <div
                key={s.session}
                className="flex items-center gap-3 text-sm py-1.5 border-b border-border-subtle last:border-0"
              >
                <span className="font-mono text-xs text-accent-bright">
                  {s.effective_agent ?? s.agent ?? "unknown"}
                </span>
                <span className="text-text-tertiary">
                  {s.project ?? "—"}
                </span>
                <span className="ml-auto text-xs text-text-tertiary">
                  {s.focus ?? s.status}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <EmptyState title="No active sessions" />
        )}
      </GlassPanel>
    </div>
  );
}

function PressureCell({
  label,
  value,
  warn,
  danger,
}: {
  label: string;
  value?: number;
  warn?: boolean;
  danger?: boolean;
}) {
  const v = value ?? 0;
  const color =
    danger && v > 0
      ? "text-status-expired"
      : warn && v > 0
        ? "text-status-stale"
        : "text-text-secondary";

  return (
    <div className="flex items-center justify-between px-4 py-3 rounded-lg border border-border-subtle bg-bg-surface/20">
      <span className="text-xs text-text-tertiary">{label}</span>
      <span className={`text-sm font-medium tabular-nums ${color}`}>{v}</span>
    </div>
  );
}
