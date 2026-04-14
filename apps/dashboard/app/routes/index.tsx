import { createFileRoute } from "@tanstack/react-router";
import { useHealth, useWorking, useInbox, useSessions } from "../lib/queries";
import { MetricCard } from "../components/ui/metric-card";
import { GlassPanel } from "../components/ui/glass-panel";
import { EmptyState } from "../components/ui/empty-state";
import { HarnessHealthPanel } from "../components/ui/harness-health";
import type { WorkingRecord, EvictedRecord } from "../lib/types";

export const Route = createFileRoute("/")({
  component: StatusDashboard,
});

/** Parse compact pipe-delimited record into displayable fields */
function parseRecord(rec: WorkingRecord) {
  const fields: Record<string, string> = {};
  for (const part of rec.record.split(" | ")) {
    const eq = part.indexOf("=");
    if (eq > 0) fields[part.slice(0, eq)] = part.slice(eq + 1);
  }
  return fields;
}

function StatusDashboard() {
  const health = useHealth();
  const working = useWorking();
  const inbox = useInbox();
  const sessions = useSessions();

  const isConnected = health.isSuccess;
  const h = health.data;
  const w = working.data;

  return (
    <div className="p-8 max-w-6xl space-y-8">
      {/* Page title */}
      <div className="flex items-center gap-3">
        <h1 className="text-2xl font-semibold tracking-tight">Status</h1>
        <span
          className={`w-2 h-2 rounded-full ${isConnected ? "bg-status-current" : "bg-status-expired"}`}
        />
        {isConnected && (
          <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
            {h?.status}
          </span>
        )}
      </div>

      {/* Metrics row */}
      <div className="grid grid-cols-4 gap-4">
        <MetricCard
          label="Total Items"
          value={h?.items ?? "—"}
          color="text-text-primary"
        />
        <MetricCard
          label="Working Memory"
          value={w?.records.length ?? "—"}
          color="text-accent-bright"
          sub={
            w
              ? `${w.used_chars}/${w.budget_chars} chars${w.truncated ? " (truncated)" : ""}`
              : undefined
          }
        />
        <MetricCard
          label="Inbox"
          value={inbox.data?.items.length ?? "—"}
          color="text-status-candidate"
        />
        <MetricCard
          label="Sessions"
          value={sessions.data?.sessions.length ?? "—"}
          color="text-status-current"
        />
      </div>

      {/* Harness bootstrap health */}
      {sessions.data?.sessions.length ? (
        <HarnessHealthPanel sessions={sessions.data.sessions} />
      ) : null}

      {/* Working memory */}
      {w && (
        <GlassPanel>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-text-secondary">
              Working Memory
            </h2>
            <div className="flex items-center gap-4 text-[11px] tracking-wide uppercase text-text-tertiary">
              <span>Budget {w.budget_chars}</span>
              <span>Used {w.used_chars}</span>
              <span>Max {w.policy.admission_limit}</span>
            </div>
          </div>

          {/* Budget bar */}
          <div className="mb-5">
            <div className="h-1 rounded-full bg-white/5 overflow-hidden">
              <div
                className="h-full rounded-full bg-accent-primary transition-all"
                style={{
                  width: `${Math.min(100, (w.used_chars / w.budget_chars) * 100)}%`,
                }}
              />
            </div>
          </div>

          {w.records.length > 0 ? (
            <div className="space-y-0">
              {w.records.map((rec) => (
                <RecordRow key={rec.id} record={rec} />
              ))}
            </div>
          ) : (
            <EmptyState
              title={isConnected ? "No working memory" : "Not connected"}
              description={isConnected ? undefined : "Start memd to see data"}
            />
          )}

          {/* Evicted */}
          {w.evicted && w.evicted.length > 0 && (
            <div className="mt-4 pt-4 border-t border-border-subtle">
              <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-2">
                Evicted ({w.evicted.length})
              </p>
              <div className="space-y-0">
                {w.evicted.slice(0, 3).map((ev) => (
                  <EvictedRow key={ev.id} record={ev} />
                ))}
              </div>
            </div>
          )}
        </GlassPanel>
      )}

      {/* Active sessions */}
      <GlassPanel>
        <h2 className="text-sm font-medium text-text-secondary mb-4">
          Active Sessions
        </h2>
        {sessions.data?.sessions.length ? (
          <div className="space-y-0">
            {sessions.data.sessions.map((s) => (
              <div
                key={s.session}
                className="flex items-center gap-3 text-sm py-2.5 border-b border-border-subtle last:border-0 hover:bg-hover transition-colors"
              >
                <span className="font-mono text-xs text-accent-bright">
                  {s.effective_agent ?? s.agent ?? "unknown"}
                </span>
                {s.hive_role && (
                  <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
                    {s.hive_role}
                  </span>
                )}
                <span className="text-text-tertiary text-xs">
                  {s.project ?? "—"}
                </span>
                {s.worker_name && (
                  <span className="ml-auto text-xs text-text-tertiary truncate max-w-48">
                    {s.worker_name}
                  </span>
                )}
                <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
                  {s.status}
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

function RecordRow({ record }: { record: WorkingRecord }) {
  const fields = parseRecord(record);
  const kind = fields.kind ?? "unknown";

  return (
    <div className="flex items-center gap-3 text-sm py-2 border-b border-border-subtle last:border-0 hover:bg-hover transition-colors">
      <span className="inline-flex items-center px-2 py-0.5 rounded text-[11px] font-medium border bg-purple-500/15 text-purple-300 border-purple-500/30">
        {kind.replace(/_/g, " ")}
      </span>
      {fields.scope && (
        <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
          {fields.scope}
        </span>
      )}
      {fields.stage && (
        <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
          {fields.stage}
        </span>
      )}
      <span className="flex-1 truncate font-mono text-xs text-text-tertiary">
        {record.id.slice(0, 8)}
      </span>
      {fields.tags && (
        <span className="text-[11px] text-text-tertiary truncate max-w-48">
          {fields.tags}
        </span>
      )}
    </div>
  );
}

function EvictedRow({ record }: { record: EvictedRecord }) {
  const fields = parseRecord(record);
  const kind = fields.kind ?? "unknown";

  return (
    <div className="flex items-center gap-2 text-xs text-text-tertiary py-1.5 hover:bg-hover transition-colors">
      <span className="opacity-60">{kind.replace(/_/g, " ")}</span>
      <span className="font-mono">{record.id.slice(0, 8)}</span>
      <span className="ml-auto truncate max-w-64 opacity-50">
        {record.reason.split(";")[0]}
      </span>
    </div>
  );
}
