import { defineConfig } from 'vite';

// Yggdrasil web client config.
// Plain TS app with iframe-hosted surface bundles. No framework.
// Public assets in clients/web/public/ are served at /.
export default defineConfig({
  root: '.',
  publicDir: 'public',
  build: {
    outDir: 'dist',
    target: 'es2022',
    sourcemap: true,
    emptyOutDir: true,
    rollupOptions: {
      output: {
        entryFileNames: 'assets/main-[hash].js',
        assetFileNames: (assetInfo) => {
          if (assetInfo.name?.endsWith('.css')) return 'assets/styles-[hash][extname]';
          return 'assets/[name]-[hash][extname]';
        },
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
    // Proxy /rpc and /kernel to host serve during dev so the web client
    // can hit them at same-origin without CORS hassle.
    proxy: {
      '/rpc': {
        target: 'http://127.0.0.1:8787',
        changeOrigin: false,
      },
      '/kernel': {
        target: 'http://127.0.0.1:8787',
        changeOrigin: false,
      },
    },
  },
});
