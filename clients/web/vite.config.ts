import { defineConfig } from 'vite';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

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
    fs: {
      // Allow serving files from the sibling YdlTavern repo for surface bundles.
      allow: [
        '..',
        resolve(__dirname, '../../../YdlTavern'),
      ],
    },
  },
  // Dev-only middleware for demo surface bundles. Production should expose
  // package assets through a host-owned same-origin static route.
  plugins: [
    {
      name: 'ydltavern-bundle-server',
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          if (!req.url?.startsWith('/surface-bundles/ydltavern/')) {
            next();
            return;
          }

          const requestPath = req.url.split('?')[0] ?? '';
          const relativePath = requestPath.replace('/surface-bundles/ydltavern/', '');
          const filePath = resolve(__dirname, '../../../YdlTavern/packages/ydltavern-surface/dist', relativePath);

          readFile(filePath)
            .then((data) => {
              if (relativePath.endsWith('.mjs') || relativePath.endsWith('.js')) {
                res.setHeader('Content-Type', 'application/javascript');
              } else if (relativePath.endsWith('.css')) {
                res.setHeader('Content-Type', 'text/css');
              } else if (relativePath.endsWith('.map')) {
                res.setHeader('Content-Type', 'application/json');
              }
              res.end(data);
            })
            .catch(() => {
              res.statusCode = 404;
              res.end('Not found');
            });
        });
      },
    },
  ],
});
