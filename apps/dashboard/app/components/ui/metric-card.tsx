import { GlassPanel } from "./glass-panel";

interface MetricCardProps {
  label: string;
  value: string | number;
  color?: string;
  sub?: string;
}

export function MetricCard({
  label,
  value,
  color = "text-text-primary",
  sub,
}: MetricCardProps) {
  return (
    <GlassPanel hover>
      <p className="text-sm text-text-secondary">{label}</p>
      <p className={`text-3xl font-semibold mt-2 tabular-nums ${color}`}>
        {value}
      </p>
      {sub && <p className="text-xs text-text-tertiary mt-1">{sub}</p>}
    </GlassPanel>
  );
}
