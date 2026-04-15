import { useEntityLinks } from "../../lib/queries";
import { GlassPanel } from "../ui/glass-panel";
import { KindBadge, ConfidenceBar } from "../ui/badge";
import type { MemoryEntityRecord } from "../../lib/types";

export function EntityDetail({
  entity,
  onClose,
}: {
  entity: MemoryEntityRecord | null;
  onClose: () => void;
}) {
  const { data } = useEntityLinks(
    entity ? { entity_id: entity.id } : undefined,
  );

  if (!entity) return null;

  return (
    <div className="w-80 shrink-0 border-l border-border-subtle bg-bg-surface/60 overflow-y-auto">
      <div className="p-4 border-b border-border-subtle flex items-center justify-between">
        <h3 className="text-sm font-medium text-text-primary truncate">
          {entity.name}
        </h3>
        <button
          onClick={onClose}
          className="text-text-tertiary hover:text-text-primary text-xs"
        >
          ✕
        </button>
      </div>

      <div className="p-4 space-y-4">
        {entity.kind && <KindBadge kind={entity.kind as any} />}

        <div className="text-xs space-y-2">
          <div>
            <span className="text-text-tertiary uppercase tracking-wide">
              ID
            </span>
            <p className="font-mono text-text-secondary">
              {entity.id.slice(0, 12)}
            </p>
          </div>
          {entity.project && (
            <div>
              <span className="text-text-tertiary uppercase tracking-wide">
                Project
              </span>
              <p className="text-text-secondary">{entity.project}</p>
            </div>
          )}
        </div>

        {data && data.links.length > 0 && (
          <GlassPanel padding="sm">
            <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-2">
              Links ({data.links.length})
            </p>
            <div className="space-y-1.5">
              {data.links.map((link) => (
                <div key={link.id} className="flex items-center gap-2 text-xs">
                  <span className="text-accent-bright font-mono">
                    {(link.source_entity_id === entity.id
                      ? link.target_name
                      : link.source_name
                    )?.slice(0, 20) ?? "?"}
                  </span>
                  <span className="text-text-tertiary">{link.relation}</span>
                  <ConfidenceBar value={link.confidence} />
                </div>
              ))}
            </div>
          </GlassPanel>
        )}
      </div>
    </div>
  );
}
