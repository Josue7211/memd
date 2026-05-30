import { createFileRoute } from "@tanstack/react-router";
import { useHealth, useWorking, useInbox, useSessions, useTasks } from "../lib/queries";
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
  const tasks = useTasks();

  const isConnected = health.isSuccess;
  const h = health.data;
  const w = working.data;
  const activeSessions = sessions.data?.sessions.filter((s) => s.status !== "retired") ?? [];
  const openTasks =
    tasks.data?.tasks.filter((task) => !["done", "closed", "retired"].includes(task.status)) ??
    [];
  const liveHarnesses = new Set(
    activeSessions.map((s) =>
      (s.effective_agent ?? s.agent ?? "unknown").toLowerCase(),
    ),
  );
  const readiness = [
    {
      label: "Server",
      ok: isConnected,
      detail: isConnected ? h?.status ?? "online" : "offline",
    },
    {
      label: "Working Set",
      ok: Boolean(w && !w.truncated),
      detail: w ? `${w.used_chars}/${w.budget_chars}` : "missing",
    },
    {
      label: "Codex",
      ok: Array.from(liveHarnesses).some((agent) => agent.includes("codex")),
      detail: "harness",
    },
    {
      label: "Group Work",
      ok: activeSessions.length > 1,
      detail: `${activeSessions.length} sessions`,
    },
  ];
  const readyCount = readiness.filter((item) => item.ok).length;
  const budgetPct = w ? Math.min(100, Math.round((w.used_chars / w.budget_chars) * 100)) : 0;
  const inboxCount = inbox.data?.items.length ?? 0;
  const staleCount = h?.pressure?.stale ?? 0;
  const expiredCount = h?.pressure?.expired ?? 0;

  return (
    <div className="p-8 max-w-7xl space-y-8">
      <div className="grid gap-6 lg:grid-cols-[1.25fr_0.75fr]">
        <section className="rounded-lg border border-border-subtle bg-bg-surface p-6">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <p className="text-[11px] tracking-wide uppercase text-text-tertiary">
                V20 Evidence Ops
              </p>
              <h1 className="mt-2 text-3xl font-semibold tracking-tight">
                memd control center
              </h1>
              <p className="mt-3 max-w-2xl text-sm leading-6 text-text-secondary">
                Runtime status is local evidence only. Release proof still
                requires real users, harness pairs, devices, auditor review,
                and third-party replay packets.
              </p>
            </div>
            <div className="rounded-lg border border-border-subtle bg-bg-primary px-4 py-3 text-right">
              <p className="text-[11px] tracking-wide uppercase text-text-tertiary">
                readiness
              </p>
              <p className="mt-1 text-2xl font-semibold text-status-current">
                {readyCount}/{readiness.length}
              </p>
            </div>
          </div>

          <div className="mt-6 grid gap-3 md:grid-cols-4">
            {readiness.map((item) => (
              <div
                key={item.label}
                className={`rounded-lg border px-3 py-3 ${
                  item.ok
                    ? "border-emerald-500/25 bg-emerald-500/10"
                    : "border-red-500/25 bg-red-500/10"
                }`}
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="text-xs font-medium text-text-secondary">
                    {item.label}
                  </span>
                  <span
                    className={`h-2 w-2 rounded-full ${
                      item.ok ? "bg-status-current" : "bg-status-expired"
                    }`}
                  />
                </div>
                <p className="mt-2 truncate font-mono text-xs text-text-tertiary">
                  {item.detail}
                </p>
              </div>
            ))}
          </div>
        </section>

        <V20GateCard />
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
          color={budgetPct > 90 ? "text-status-expired" : "text-accent-bright"}
          sub={
            w
              ? `${budgetPct}% budget${w.truncated ? " (truncated)" : ""}`
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
          sub={activeSessions.length ? `${activeSessions.length} active` : undefined}
        />
      </div>

      {/* Pressure metrics */}
      {h?.pressure && (
        <div className="grid grid-cols-4 gap-4">
          <MetricCard
            label="Eval Score"
            value={h.eval_score != null ? `${Math.round(h.eval_score)}` : "—"}
            color={
              h.eval_score == null
                ? "text-text-tertiary"
                : h.eval_score >= 80
                  ? "text-status-current"
                  : h.eval_score >= 50
                    ? "text-status-stale"
                    : "text-status-expired"
            }
          />
          <MetricCard
            label="Candidates"
            value={h.pressure.candidates}
            color="text-status-candidate"
          />
          <MetricCard
            label="Stale"
            value={h.pressure.stale}
            color="text-status-stale"
          />
          <MetricCard
            label="Expired"
            value={h.pressure.expired}
            color="text-status-expired"
          />
        </div>
      )}

      {/* Harness bootstrap health */}
      <HarnessHealthPanel sessions={sessions.data?.sessions ?? []} />

      <div className="grid gap-6 lg:grid-cols-2">
        <GlassPanel>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-text-secondary">
              Evidence Pressure
            </h2>
            <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
              live blockers
            </span>
          </div>
          <div className="grid grid-cols-3 gap-3">
            <PressurePill label="Inbox" value={inboxCount} tone="blue" />
            <PressurePill label="Stale" value={staleCount} tone="amber" />
            <PressurePill label="Expired" value={expiredCount} tone="red" />
          </div>
          <div className="mt-5 space-y-2 text-xs text-text-secondary">
            <p>Next evidence note: 2026-05-13.</p>
            <p>No `1.0.0` tag until real-user, device, auditor, and third-party replay packets exist.</p>
            <p>This dashboard does not claim those gates are complete.</p>
          </div>
        </GlassPanel>

        <GlassPanel>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-text-secondary">
              Open Hive Work
            </h2>
            <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
              {openTasks.length} open
            </span>
          </div>
          {openTasks.length > 0 ? (
            <div className="space-y-0">
              {openTasks.slice(0, 5).map((task) => (
                <div
                  key={task.id}
                  className="flex items-center gap-3 border-b border-border-subtle py-2.5 text-sm last:border-0"
                >
                  <span className="min-w-16 rounded border border-border-subtle bg-bg-primary px-2 py-0.5 text-center text-[11px] uppercase text-text-tertiary">
                    {task.status}
                  </span>
                  <span className="min-w-0 flex-1 truncate text-text-primary">
                    {task.title}
                  </span>
                  {task.assigned_to && (
                    <span className="max-w-32 truncate font-mono text-xs text-text-tertiary">
                      {task.assigned_to}
                    </span>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <EmptyState title="No open hive tasks" />
          )}
        </GlassPanel>
      </div>

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

function V20GateCard() {
  const gates = [
    "3 real users",
    "3 harness-user pairs",
    "3 devices",
    "auditor review packet",
    "third-party replay packet",
    "weekly evidence notes",
  ];

  return (
    <section className="rounded-lg border border-border-subtle bg-bg-surface p-6">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-[11px] tracking-wide uppercase text-text-tertiary">
            1.0.0 gate
          </p>
          <h2 className="mt-2 text-lg font-semibold">Real proof only</h2>
        </div>
        <span className="rounded border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-[11px] font-medium uppercase text-amber-300">
          pending
        </span>
      </div>
      <div className="mt-5 space-y-2">
        {gates.map((gate) => (
          <div key={gate} className="flex items-center gap-2 text-sm">
            <span className="h-1.5 w-1.5 rounded-full bg-amber-400" />
            <span className="text-text-secondary">{gate}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

function PressurePill({
  label,
  value,
  tone,
}: {
  label: string;
  value: number;
  tone: "blue" | "amber" | "red";
}) {
  const toneClass = {
    blue: "border-blue-500/25 bg-blue-500/10 text-blue-300",
    amber: "border-amber-500/25 bg-amber-500/10 text-amber-300",
    red: "border-red-500/25 bg-red-500/10 text-red-300",
  }[tone];

  return (
    <div className={`rounded-lg border px-3 py-3 ${toneClass}`}>
      <p className="text-[11px] tracking-wide uppercase opacity-80">{label}</p>
      <p className="mt-1 text-xl font-semibold">{value}</p>
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
