import type { ReactNode } from "react";

interface GlassPanelProps {
  children: ReactNode;
  className?: string;
  hover?: boolean;
  padding?: "none" | "sm" | "md" | "lg";
}

const paddings = {
  none: "",
  sm: "p-3",
  md: "p-5",
  lg: "p-6",
};

export function GlassPanel({
  children,
  className = "",
  hover = false,
  padding = "md",
}: GlassPanelProps) {
  return (
    <div
      className={`
        rounded-xl border border-border-subtle bg-bg-surface/40
        ${paddings[padding]}
        ${hover ? "hover:border-border-active hover:shadow-[0_0_20px_var(--color-accent-glow)] transition-all" : ""}
        ${className}
      `}
    >
      {children}
    </div>
  );
}
