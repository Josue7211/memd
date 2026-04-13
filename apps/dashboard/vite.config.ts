import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
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
      "/healthz": "http://localhost:3080",
      "/memory": "http://localhost:3080",
      "/atlas": "http://localhost:3080",
      "/procedures": "http://localhost:3080",
      "/coordination": "http://localhost:3080",
      "/hive": "http://localhost:3080",
    },
  },
});
