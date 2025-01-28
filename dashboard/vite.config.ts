import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";
import { visualizer } from "rollup-plugin-visualizer";
import compression from "vite-plugin-compression";
import type { Plugin } from "vite";

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

export default defineConfig({
  plugins: [
    handlePureAnnotations,
    react(),
    VitePWA({
      registerType: "autoUpdate",
      manifest: {
        name: "listen",
        short_name: "listen",
        theme_color: "#A855F7",
        icons: [
          {
            src: "/listen-more.png",
            sizes: "192x192",
            type: "image/png",
          },
        ],
      },
      workbox: {
        maximumFileSizeToCacheInBytes: 3000000,
      },
    }),
    visualizer(),
    compression(),
  ],
  build: {
    target: "esnext",
    minify: "esbuild",
    sourcemap: false,
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("node_modules")) {
            if (id.includes("react")) {
              return "vendor-react";
            }
            if (id.includes("@privy-io")) {
              return "vendor-privy";
            }
            if (id.includes("@coinbase")) {
              return "vendor-coinbase";
            }
            if (id.includes("viem")) {
              return "vendor-viem";
            }
            if (id.includes("@walletconnect")) {
              return "vendor-walletconnect";
            }
            if (id.includes("@ethersproject")) {
              return "vendor-etheresproject";
            }
            if (id.includes("@noble")) {
              return "vendor-noble";
            }
            if (id.includes("lodash")) {
              return "vendor-lodash";
            }
            if (id.includes("@solana")) {
              return "vendor-solana";
            }
            if (id.includes("@tanstack")) {
              return "vendor-tanstack";
            }
            return "vendor";
          }
        },
      },
    },
  },
});
