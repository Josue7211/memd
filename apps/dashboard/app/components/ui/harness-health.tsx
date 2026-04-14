import type { HiveSessionRecord } from "../../lib/types";
import { GlassPanel } from "./glass-panel";

const WAKE_TTL_SECONDS = 120;

interface HarnessGroup {
  harness: string;
  sessions: HiveSessionRecord[];
  lastWakeAt: Date | null;
  isBootstrapped: boolean;
  isStale: boolean;
}

function deriveHarness(session: HiveSessionRecord): string {
  const agent = session.effective_agent ?? session.agent ?? "";
  const lower = agent.toLowerCase();
  if (lower.includes("claude-code") || lower.includes("claude_code"))
    return "claude-code";
  if (lower.includes("codex")) return "codex";
  if (lower.includes("openclaw") || lower.includes("claw")) return "openclaw";
  if (lower.includes("hermes")) return "hermes";
  if (lower.includes("opencode")) return "opencode";
  if (lower.includes("agent-zero") || lower.includes("agent_zero"))
    return "agent-zero";
  return agent || "unknown";
}

function groupByHarness(sessions: HiveSessionRecord[]): HarnessGroup[] {
  const map = new Map<string, HiveSessionRecord[]>();
  for (const s of sessions) {
    const h = deriveHarness(s);
    if (!map.has(h)) map.set(h, []);
    map.get(h)!.push(s);
  }

  const now = Date.now();
  return Array.from(map.entries()).map(([harness, sessions]) => {
    const wakes = sessions
      .filter((s) => s.last_wake_at)
      .map((s) => new Date(s.last_wake_at!).getTime());
    const latestWake = wakes.length > 0 ? Math.max(...wakes) : null;
    const isBootstrapped = latestWake !== null;
    const isStale =
      latestWake !== null && now - latestWake > WAKE_TTL_SECONDS * 1000;

    return {
      harness,
      sessions,
      lastWakeAt: latestWake ? new Date(latestWake) : null,
      isBootstrapped,
      isStale,
    };
  });
}

function timeAgo(date: Date): string {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function BootstrapDot({ group }: { group: HarnessGroup }) {
  if (!group.isBootstrapped) {
    return (
      <span
        className="w-2 h-2 rounded-full bg-red-400"
        title="Never bootstrapped"
      />
    );
  }
  if (group.isStale) {
    return (
      <span
        className="w-2 h-2 rounded-full bg-amber-400"
        title="Bootstrap stale"
      />
    );
  }
  return (
    <span
      className="w-2 h-2 rounded-full bg-emerald-400"
      title="Bootstrapped"
    />
  );
}

export function HarnessHealthPanel({
  sessions,
}: {
  sessions: HiveSessionRecord[];
}) {
  const groups = groupByHarness(sessions);

  if (groups.length === 0) return null;

  const totalBootstrapped = groups.filter((g) => g.isBootstrapped).length;
  const totalStale = groups.filter((g) => g.isStale).length;
  const totalUnbooted = groups.filter((g) => !g.isBootstrapped).length;

  return (
    <GlassPanel>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-medium text-text-secondary">
          Harness Bootstrap Health
        </h2>
        <div className="flex items-center gap-4 text-[11px] tracking-wide uppercase text-text-tertiary">
          <span className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
            {totalBootstrapped - totalStale} live
          </span>
          {totalStale > 0 && (
            <span className="flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 rounded-full bg-amber-400" />
              {totalStale} stale
            </span>
          )}
          {totalUnbooted > 0 && (
            <span className="flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 rounded-full bg-red-400" />
              {totalUnbooted} unbooted
            </span>
          )}
        </div>
      </div>

      <div className="space-y-0">
        {groups.map((group) => (
          <div
            key={group.harness}
            className="flex items-center gap-3 text-sm py-2.5 border-b border-border-subtle last:border-0 hover:bg-hover transition-colors"
          >
            <BootstrapDot group={group} />
            <span className="font-mono text-xs text-accent-bright min-w-24">
              {group.harness}
            </span>
            <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
              {group.sessions.length} session
              {group.sessions.length !== 1 ? "s" : ""}
            </span>
            <span className="flex-1" />
            {group.lastWakeAt ? (
              <span className="text-xs text-text-tertiary">
                woke {timeAgo(group.lastWakeAt)}
              </span>
            ) : (
              <span className="text-xs text-red-400/80">never woke</span>
            )}
            <span
              className={`text-[11px] font-medium px-2 py-0.5 rounded border ${
                !group.isBootstrapped
                  ? "bg-red-500/15 text-red-300 border-red-500/30"
                  : group.isStale
                    ? "bg-amber-500/15 text-amber-300 border-amber-500/30"
                    : "bg-emerald-500/15 text-emerald-300 border-emerald-500/30"
              }`}
            >
              {!group.isBootstrapped
                ? "unbooted"
                : group.isStale
                  ? "stale"
                  : "live"}
            </span>
          </div>
        ))}
      </div>
    </GlassPanel>
  );
}
