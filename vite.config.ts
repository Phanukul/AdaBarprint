import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/postcss";
import path from "path";
import { build as bundle } from 'esbuild';

export default defineConfig({
  plugins: [react(), {
    name: 'legacy-print-page',
    async generateBundle() {
      const result = await bundle({
        entryPoints: ['frontend/legacyPrintPage.tsx'], bundle: true, minify: true,
        write: false, format: 'iife', define: { 'process.env.NODE_ENV': '"production"' },
      });
      this.emitFile({ type: 'asset', fileName: 'legacy-print.js', source: result.outputFiles[0].text });
    },
  }],
  css: {
    postcss: {
      plugins: [tailwindcss()],
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./frontend"),
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:3000",
        changeOrigin: true,
      },
    },
  },
  build: {
    target: ["es2020", "chrome83"],
    outDir: "dist",
    emptyOutDir: true,
  },
});
