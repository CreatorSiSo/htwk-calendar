import { defineConfig } from "astro/config";

import tailwind from "@astrojs/tailwind";
import preact from "@astrojs/preact";

// https://astro.build/config
export default defineConfig({
  site: "https://htwk-calendar-16672e5a5a3b.herokuapp.com",
  integrations: [tailwind(), preact()],
});
