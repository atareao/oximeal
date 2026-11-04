import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      // El flujo OIDC (`/auth/login`, `/auth/me`, `/auth/logout`) vive fuera de
      // `/api`; sin este proxy el SPA de Vite respondería `index.html`.
      '/auth': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    manifest: true,
    rollupOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: 'react',
              test: /node_modules[\\/](react|react-dom|scheduler|react-router)/,
              priority: 30,
            },
            {
              name: 'query',
              test: /node_modules[\\/]@tanstack/,
              priority: 20,
            },
            {
              name: 'antd',
              test: /node_modules[\\/](antd|@ant-design|@rc-component|rc-)/,
              priority: 10,
              includeDependenciesRecursively: false,
            },
          ],
        },
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: false,
    setupFiles: ['./src/test/setup.ts'],
    // Con `css: false`, vitest stubbea todo id que case CSS_LANGS_RE —incluido
    // `*.css?raw`— devolviendo "". Se habilitan *solo* los ids `?raw` para poder
    // leer el CSS en los tests de accesibilidad sin tocar el sistema de ficheros.
    css: { include: [/\.css\?raw$/] },
    restoreMocks: true,
  },
});
