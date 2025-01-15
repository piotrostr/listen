import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
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
    }),
  ],
});
