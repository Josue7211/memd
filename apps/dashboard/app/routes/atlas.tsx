import { createFileRoute } from "@tanstack/react-router";
import { useState, useCallback, useRef, useEffect } from "react";
import { useAtlasRegions, useAtlasExplore } from "../lib/queries";
import { EmptyState } from "../components/ui/empty-state";
import { GraphToggle } from "../components/graph/graph-toggle";
import { ForceGraphWrapper } from "../components/graph/force-graph-wrapper";
import { useGraphMode } from "../components/graph/use-graph-mode";
import type { GraphNode, GraphLink } from "../components/graph/force-graph-wrapper";
import type { AtlasRegion, AtlasNode, AtlasLink } from "../lib/types";

export const Route = createFileRoute("/atlas")({
  component: AtlasPage,
});

function AtlasPage() {
  const { mode, toggle } = useGraphMode();
  const [selectedRegion, setSelectedRegion] = useState<string | null>(null);
  const [_selectedEntity, setSelectedEntity] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dims, setDims] = useState({ width: 800, height: 600 });

  const regions = useAtlasRegions();
  const explore = useAtlasExplore(selectedRegion ?? "");

  // Measure container
  useEffect(() => {
    if (!containerRef.current) return;
    const obs = new ResizeObserver((entries) => {
      const { width, height } = entries[0].contentRect;
      setDims({ width, height: Math.max(400, height) });
    });
    obs.observe(containerRef.current);
    return () => obs.disconnect();
  }, []);

  // Build graph data from regions or region detail
  const { nodes, links } = buildGraphData(
    regions.data?.regions,
    explore.data,
    selectedRegion,
  );

  const handleNodeClick = useCallback(
    (node: GraphNode) => {
      if (node.type === "region") {
        setSelectedRegion(node.id);
        setSelectedEntity(null);
      } else {
        setSelectedEntity(node.id);
      }
    },
    [],
  );

  return (
    <div className="flex flex-col h-screen">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-8 py-4 border-b border-border-subtle">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-semibold tracking-tight">Atlas</h1>
          {selectedRegion && explore.data && (
            <>
              <span className="text-text-tertiary">›</span>
              <span className="text-sm text-accent-bright">
                {explore.data.region.name}
              </span>
              <button
                onClick={() => {
                  setSelectedRegion(null);
                  setSelectedEntity(null);
                }}
                className="text-xs text-text-tertiary hover:text-text-primary transition-colors"
              >
                ← Back to regions
              </button>
            </>
          )}
        </div>
        <GraphToggle mode={mode} onToggle={toggle} />
      </div>

      {/* Graph area */}
      <div ref={containerRef} className="flex-1 relative">
        {regions.isLoading && (
          <div className="absolute inset-0 flex items-center justify-center text-text-tertiary">
            Loading...
          </div>
        )}
        {regions.data && nodes.length === 0 && (
          <EmptyState
            title="No atlas regions"
            description="Store more memories to generate regions"
          />
        )}
        {nodes.length > 0 && (
          <ForceGraphWrapper
            mode={mode}
            nodes={nodes}
            links={links}
            width={dims.width}
            height={dims.height}
            onNodeClick={handleNodeClick}
          />
        )}
      </div>

      {/* Region sidebar */}
      {selectedRegion && explore.data && (
        <RegionSidebar
          region={explore.data.region}
          nodeCount={explore.data.nodes.length}
          onClose={() => setSelectedRegion(null)}
        />
      )}
    </div>
  );
}

function buildGraphData(
  regions?: AtlasRegion[],
  exploreData?: {
    region: AtlasRegion;
    nodes: AtlasNode[];
    links: AtlasLink[];
  },
  selectedRegion?: string | null,
): { nodes: GraphNode[]; links: GraphLink[] } {
  // Drill-down mode: show entities within region
  if (selectedRegion && exploreData) {
    const nodes: GraphNode[] = exploreData.nodes.map((n) => ({
      id: n.entity_id,
      name: n.name || `Entity ${n.entity_id.slice(0, 8)}`,
      kind: n.kind,
      type: "entity",
      confidence: n.confidence,
    }));
    const links: GraphLink[] = exploreData.links.map((l) => ({
      source: l.source_entity_id,
      target: l.target_entity_id,
      relation: l.relation,
    }));
    return { nodes, links };
  }

  // Top level: show regions as nodes
  if (!regions) return { nodes: [], links: [] };

  const nodes: GraphNode[] = regions.map((r) => ({
    id: r.id,
    name: r.name,
    kind: "region",
    type: "region",
    nodeCount: r.node_count,
    confidence: 1,
  }));

  return { nodes, links: [] };
}

function RegionSidebar({
  region,
  nodeCount,
  onClose,
}: {
  region: AtlasRegion;
  nodeCount: number;
  onClose: () => void;
}) {
  return (
    <div className="absolute right-0 top-0 w-72 h-full border-l border-border-subtle bg-bg-surface/90 overflow-y-auto">
      <div className="p-4 border-b border-border-subtle flex items-center justify-between">
        <h3 className="text-sm font-medium truncate">{region.name}</h3>
        <button
          onClick={onClose}
          className="text-text-tertiary hover:text-text-primary text-xs"
        >
          ✕
        </button>
      </div>
      <div className="p-4 space-y-3 text-xs">
        {region.description && (
          <p className="text-text-secondary">{region.description}</p>
        )}
        <div>
          <span className="text-text-tertiary uppercase tracking-wide">
            Entities
          </span>
          <p className="text-text-primary text-lg font-semibold">
            {nodeCount}
          </p>
        </div>
        {region.tags.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {region.tags.map((t) => (
              <span
                key={t}
                className="px-2 py-0.5 rounded text-[11px] bg-white/5 text-text-tertiary border border-white/5"
              >
                {t}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
