import { createFileRoute } from "@tanstack/react-router";
import { useState, useCallback, useRef, useEffect } from "react";
import { useEntitySearch, useEntityLinks } from "../lib/queries";
import { EmptyState } from "../components/ui/empty-state";
import { GraphToggle } from "../components/graph/graph-toggle";
import { ForceGraphWrapper } from "../components/graph/force-graph-wrapper";
import { EntityDetail } from "../components/graph/entity-detail";
import { useGraphMode } from "../components/graph/use-graph-mode";
import type { GraphNode, GraphLink } from "../components/graph/force-graph-wrapper";
import type { MemoryEntityRecord, EntityRelationKind } from "../lib/types";

export const Route = createFileRoute("/graph")({
  component: GraphPage,
});

const ALL_RELATIONS: EntityRelationKind[] = [
  "same_as",
  "derived_from",
  "supersedes",
  "contradicts",
  "related",
];

function GraphPage() {
  const { mode, toggle } = useGraphMode();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedEntity, setSelectedEntity] =
    useState<MemoryEntityRecord | null>(null);
  const [visibleRelations, setVisibleRelations] = useState<
    Set<EntityRelationKind>
  >(new Set(ALL_RELATIONS));
  const containerRef = useRef<HTMLDivElement>(null);
  const [dims, setDims] = useState({ width: 800, height: 600 });

  const search = useEntitySearch(searchQuery);
  const links = useEntityLinks();

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

  // Build nodes from search results + link endpoints
  const { nodes, graphLinks } = buildEntityGraph(
    search.data?.entities ?? [],
    links.data?.links ?? [],
    visibleRelations,
  );

  const handleNodeClick = useCallback(
    (node: GraphNode) => {
      const entity = search.data?.entities.find((e) => e.id === node.id);
      if (entity) setSelectedEntity(entity);
    },
    [search.data],
  );

  const toggleRelation = (rel: EntityRelationKind) => {
    setVisibleRelations((prev) => {
      const next = new Set(prev);
      if (next.has(rel)) next.delete(rel);
      else next.add(rel);
      return next;
    });
  };

  return (
    <div className="flex h-screen">
      <div className="flex-1 flex flex-col">
        {/* Toolbar */}
        <div className="flex items-center gap-4 px-8 py-4 border-b border-border-subtle">
          <h1 className="text-2xl font-semibold tracking-tight">Graph</h1>

          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search entities..."
            className="flex-1 max-w-md px-4 py-2 rounded-lg bg-bg-primary border border-border-subtle text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary transition-colors"
          />

          {/* Relation filters */}
          <div className="flex gap-1">
            {ALL_RELATIONS.map((rel) => (
              <button
                key={rel}
                onClick={() => toggleRelation(rel)}
                className={`px-2 py-1 rounded text-[11px] border transition-colors ${
                  visibleRelations.has(rel)
                    ? "bg-accent-primary/15 text-accent-bright border-accent-primary/40"
                    : "bg-glass text-text-tertiary border-border-subtle"
                }`}
              >
                {rel.replace(/_/g, " ")}
              </button>
            ))}
          </div>

          <GraphToggle mode={mode} onToggle={toggle} />
        </div>

        {/* Graph area */}
        <div ref={containerRef} className="flex-1 relative">
          {nodes.length === 0 && (
            <EmptyState
              title={searchQuery ? "No entities found" : "Search for entities"}
              description="Type an entity name to explore relationships"
            />
          )}
          {nodes.length > 0 && (
            <ForceGraphWrapper
              mode={mode}
              nodes={nodes}
              links={graphLinks}
              width={dims.width - (selectedEntity ? 320 : 0)}
              height={dims.height}
              onNodeClick={handleNodeClick}
            />
          )}
        </div>
      </div>

      {/* Entity detail sidebar */}
      <EntityDetail
        entity={selectedEntity}
        onClose={() => setSelectedEntity(null)}
      />
    </div>
  );
}

function buildEntityGraph(
  entities: MemoryEntityRecord[],
  allLinks: {
    source_entity_id: string;
    target_entity_id: string;
    relation: EntityRelationKind;
  }[],
  visibleRelations: Set<EntityRelationKind>,
): { nodes: GraphNode[]; graphLinks: GraphLink[] } {
  const entityIds = new Set(entities.map((e) => e.id));

  // Include entities that are link endpoints of our search results
  const relevantLinks = allLinks.filter(
    (l) =>
      visibleRelations.has(l.relation) &&
      (entityIds.has(l.source_entity_id) ||
        entityIds.has(l.target_entity_id)),
  );

  // Collect all relevant node IDs
  const allIds = new Set(entityIds);
  for (const l of relevantLinks) {
    allIds.add(l.source_entity_id);
    allIds.add(l.target_entity_id);
  }

  const nodes: GraphNode[] = [];
  for (const id of allIds) {
    const entity = entities.find((e) => e.id === id);
    nodes.push({
      id,
      name: entity?.name ?? id.slice(0, 8),
      kind: entity?.kind ?? undefined,
      type: "entity",
      confidence: 0.7,
    });
  }

  const graphLinks: GraphLink[] = relevantLinks.map((l) => ({
    source: l.source_entity_id,
    target: l.target_entity_id,
    relation: l.relation,
  }));

  return { nodes, graphLinks };
}
