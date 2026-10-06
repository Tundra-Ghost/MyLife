import { defineConfig } from "vitest/config";

// Tests run in Anchorage time so date parsing matches what Tanner sees.
process.env.TZ = "America/Anchorage";

export default defineConfig({
  test: { environment: "node", include: ["src/**/*.test.ts", "tests/**/*.test.ts"] },
});
