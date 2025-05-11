import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { visualizer } from "rollup-plugin-visualizer";
import type { Plugin } from "vite";
import { defineConfig, loadEnv } from "vite";
import compression from "vite-plugin-compression";
import { VitePWA } from "vite-plugin-pwa";

const handlePureAnnotations: Plugin = {
  name: "handle-pure-annotations",
  transform(code: string, id: string) {
    if (id.includes("@privy-io/react-auth")) {
      return {
        code: code.replace(/\/\*#__PURE__\*\//g, ""),
        map: null,
      };
    }
  },
};

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const isWorldMiniAppEnabled = env.VITE_WORLD_MINIAPP_ENABLED === "true";

  // Only include PWA plugin if World Mini App is not enabled
  const plugins = [
    handlePureAnnotations,
    TanStackRouterVite(),
    react(),
    visualizer(),
    compression(),
  ];

  if (!isWorldMiniAppEnabled) {
    plugins.push(
      VitePWA({
        registerType: "autoUpdate",
        manifest: {
          name: "listen",
          short_name: "listen",
          theme_color: "#000000",
          icons: [
            {
              src: "/listen-icon.png",
              sizes: "192x192",
              type: "image/png",
            },
          ],
        },
        workbox: {
          maximumFileSizeToCacheInBytes: 4 * 1024 * 1024,
        },
      })
    );
  }

  return {
    plugins,
    build: {
      target: "esnext",
      minify: "esbuild",
      sourcemap: false,
      chunkSizeWarningLimit: 1000,
    },
  };
});
