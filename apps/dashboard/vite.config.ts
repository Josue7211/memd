import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const MEMD_API = env.MEMD_API_URL || "http://localhost:3080";

  return {
    base: "/dashboard/",
    plugins: [
      tailwindcss(),
      TanStackRouterVite({
        routesDirectory: "./app/routes",
        generatedRouteTree: "./app/routeTree.gen.ts",
      }),
      react(),
    ],
    server: {
      port: 5173,
      proxy: {
        "/healthz": MEMD_API,
        "/memory": MEMD_API,
        "/atlas": MEMD_API,
        "/procedures": MEMD_API,
        "/coordination": MEMD_API,
        "/hive": MEMD_API,
      },
    },
  };
});
