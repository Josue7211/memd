import { useRef, useCallback, useEffect } from "react";
import ForceGraph2DComp from "react-force-graph-2d";
import ForceGraph3DComp from "react-force-graph-3d";
import type { GraphMode } from "./use-graph-mode";
import {
  FORCE_CONFIG,
  nodeRadius,
  KIND_COLORS,
  RELATION_COLORS,
} from "./graph-constants";

export interface GraphNode {
  id: string;
  name: string;
  kind?: string;
  type?: string;
  nodeCount?: number;
  confidence?: number;
}

export interface GraphLink {
  source: string;
  target: string;
  relation?: string;
}

interface ForceGraphWrapperProps {
  mode: GraphMode;
  nodes: GraphNode[];
  links: GraphLink[];
  width: number;
  height: number;
  onNodeClick?: (node: GraphNode) => void;
}

export function ForceGraphWrapper({
  mode,
  nodes,
  links,
  width,
  height,
  onNodeClick,
}: ForceGraphWrapperProps) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const fgRef = useRef<any>(null);

  const graphData = { nodes, links };

  const nodeColor = useCallback(
    (node: GraphNode) =>
      KIND_COLORS[node.kind ?? node.type ?? ""] ?? "#555570",
    [],
  );

  const linkColor = useCallback(
    (link: GraphLink) => RELATION_COLORS[link.relation ?? ""] ?? "#555570",
    [],
  );

  const nodeVal = useCallback((node: GraphNode) => nodeRadius(node), []);

  useEffect(() => {
    if (fgRef.current) {
      fgRef.current.d3Force("charge")?.strength(FORCE_CONFIG.charge);
    }
  }, [mode]);

  const commonProps = {
    ref: fgRef,
    graphData,
    width,
    height,
    nodeColor,
    nodeVal,
    nodeLabel: (n: GraphNode) => n.name,
    linkColor,
    linkDirectionalArrowLength: 4,
    linkDirectionalArrowRelPos: 1,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onNodeClick: onNodeClick as any,
    warmupTicks: FORCE_CONFIG.warmupTicks,
    d3AlphaDecay: FORCE_CONFIG.alphaDecay,
  };

  if (mode === "3d") {
    return <ForceGraph3DComp {...commonProps} />;
  }

  return <ForceGraph2DComp {...commonProps} />;
}
