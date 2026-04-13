import type { MemoryKind, MemoryStage, MemoryStatus, ProcedureStatus } from "../../lib/types";

// ── Kind Badge ───────────────────────────────────────────────────────────────

const kindColors: Record<MemoryKind, string> = {
  fact: "bg-purple-500/15 text-purple-300 border-purple-500/30",
  decision: "bg-violet-500/15 text-violet-300 border-violet-500/30",
  preference: "bg-indigo-500/15 text-indigo-300 border-indigo-500/30",
  runbook: "bg-sky-500/15 text-sky-300 border-sky-500/30",
  procedural: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
  self_model: "bg-amber-500/15 text-amber-300 border-amber-500/30",
  topology: "bg-cyan-500/15 text-cyan-300 border-cyan-500/30",
  status: "bg-zinc-500/15 text-zinc-400 border-zinc-500/30",
  live_truth: "bg-rose-500/15 text-rose-300 border-rose-500/30",
  pattern: "bg-fuchsia-500/15 text-fuchsia-300 border-fuchsia-500/30",
  constraint: "bg-red-500/15 text-red-300 border-red-500/30",
};

export function KindBadge({ kind }: { kind: MemoryKind }) {
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded text-[11px] font-medium border ${kindColors[kind] ?? "bg-zinc-500/15 text-zinc-400 border-zinc-500/30"}`}
    >
      {kind.replace("_", " ")}
    </span>
  );
}

// ── Stage Badge ──────────────────────────────────────────────────────────────

const stageColors: Record<MemoryStage, string> = {
  candidate: "bg-blue-500/15 text-blue-300 border-blue-500/30",
  canonical: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
};

export function StageBadge({ stage }: { stage: MemoryStage }) {
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded text-[11px] font-medium border ${stageColors[stage]}`}
    >
      {stage}
    </span>
  );
}

// ── Status Indicator ─────────────────────────────────────────────────────────

const statusColors: Record<MemoryStatus, string> = {
  active: "bg-emerald-400",
  stale: "bg-amber-400",
  superseded: "bg-zinc-500",
  contested: "bg-orange-400",
  expired: "bg-red-400",
};

export function StatusDot({ status }: { status: MemoryStatus }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-[11px] text-text-secondary">
      <span className={`w-1.5 h-1.5 rounded-full ${statusColors[status]}`} />
      {status}
    </span>
  );
}

// ── Procedure Status Badge ───────────────────────────────────────────────────

const procStatusColors: Record<ProcedureStatus, string> = {
  candidate: "bg-blue-500/15 text-blue-300 border-blue-500/30",
  promoted: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
  retired: "bg-zinc-500/15 text-zinc-400 border-zinc-500/30",
};

export function ProcedureStatusBadge({ status }: { status: ProcedureStatus }) {
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded text-[11px] font-medium border ${procStatusColors[status]}`}
    >
      {status}
    </span>
  );
}

// ── Confidence Bar ───────────────────────────────────────────────────────────

export function ConfidenceBar({ value }: { value: number }) {
  const pct = Math.round(value * 100);
  const color =
    pct >= 80 ? "bg-emerald-400" : pct >= 50 ? "bg-amber-400" : "bg-red-400";

  return (
    <div className="flex items-center gap-2">
      <div className="flex-1 h-1 rounded-full bg-white/5">
        <div
          className={`h-1 rounded-full ${color} transition-all`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="text-[11px] text-text-tertiary tabular-nums">
        {pct}%
      </span>
    </div>
  );
}
