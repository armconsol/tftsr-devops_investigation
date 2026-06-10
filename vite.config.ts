import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  resolve: {
    alias: { "@": path.resolve(__dirname, "./src") },
  },
  worker: {
    format: "es",
  },
  optimizeDeps: {
    include: [
      "ansi-to-react",
      "monaco-editor/esm/vs/language/json/json.worker",
      "monaco-editor/esm/vs/editor/editor.worker",
    ],
  },
}));
