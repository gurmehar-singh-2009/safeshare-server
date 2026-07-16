import { defineConfig } from "vite";
import htmlMinifier from "vite-plugin-html-minifier";

export default defineConfig({
  plugins: [
    htmlMinifier({
      minify: true,
    }),
  ],
  build: {
    minify: true,
    rollupOptions: {
      output: {
        entryFileNames: `assets/[name].js`,
        chunkFileNames: `assets/[name].js`,
        assetFileNames: `assets/[name].[ext]`,
      },
    },
  },
});
