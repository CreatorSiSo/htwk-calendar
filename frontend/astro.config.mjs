import { defineConfig } from "astro/config";

import tailwind from "@astrojs/tailwind";
import preact from "@astrojs/preact";

// https://astro.build/config
export default defineConfig({
  // site: "http://localhost:5000",
  // site: "http://192.168.178.154:5000",
  // site: "https://htwk-calendar-16672e5a5a3b.herokuapp.com",
  site: "https://calendar.htwk.app",
  integrations: [tailwind(), preact({ compat: true })],
  vite: {
    plugins: [],
    build: {
      rollupOptions: {
        output: {
          manualChunks: (path, meta) => {
            if (path.includes("luxon")) return "luxon";
            if (path.includes("fullcalendar")) return "fullcalendar";
          },
        },
      },
    },
  },
});
