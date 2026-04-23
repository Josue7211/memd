import { useState, useCallback } from "react";

export type GraphMode = "2d" | "3d";

const STORAGE_KEY = "memd-graph-mode";

function readMode(): GraphMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === "3d" ? "3d" : "2d";
  } catch {
    return "2d";
  }
}

export function useGraphMode() {
  const [mode, setModeState] = useState<GraphMode>(readMode);

  const setMode = useCallback((m: GraphMode) => {
    setModeState(m);
    try {
      localStorage.setItem(STORAGE_KEY, m);
    } catch {
      /* noop */
    }
  }, []);

  const toggle = useCallback(() => {
    setModeState((prev) => {
      const next = prev === "2d" ? "3d" : "2d";
      try {
        localStorage.setItem(STORAGE_KEY, next);
      } catch {
        /* noop */
      }
      return next;
    });
  }, []);

  return { mode, setMode, toggle } as const;
}
