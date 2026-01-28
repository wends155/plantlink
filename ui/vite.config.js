import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
import viteCompression from 'vite-plugin-compression';

// https://vite.dev/config/
export default defineConfig(({ command, mode }) => {
  const isProduction = mode === 'production' || command === 'build';

  return {
    plugins: [
      svelte(),
      isProduction && viteCompression({
        algorithm: 'gzip',
        ext: '.gz',
      })
    ].filter(Boolean),
    build: {
      minify: isProduction, // Minify only in production
      sourcemap: !isProduction, // Sourcemaps only in dev/debug
    }
  };
});
