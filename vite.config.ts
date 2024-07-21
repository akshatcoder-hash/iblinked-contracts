import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  test: {
    testTimeout: 100000,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./tests"),
    },
  },
});
