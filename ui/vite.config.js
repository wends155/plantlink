import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import viteCompression from 'vite-plugin-compression';

export default defineConfig(({ command, mode }) => {
  const isProduction = mode === 'production' || command === 'build';

  return {
    plugins: [
      svelte(),
      isProduction && viteCompression({ algorithm: 'gzip', ext: '.gz' }),
    ].filter(Boolean),

    esbuild: {
      pure: isProduction
        ? ['console.log', 'console.info', 'console.debug']
        : [],
    },

    build: {
      minify: isProduction,
      sourcemap: !isProduction,
      chunkSizeWarningLimit: 500,

      rollupOptions: {
        output: {
          manualChunks: {
            // Split large dependencies into separate chunks
            'codemirror': [
              'codemirror',
              '@codemirror/lang-rust',
              '@codemirror/theme-one-dark',
              '@codemirror/view',
              '@codemirror/language',
              '@lezer/highlight',
            ],
            'flow': ['@xyflow/svelte'],
            'icons': ['lucide-svelte'],
          },
        }
      },
    },

    optimizeDeps: {
      include: ['@xyflow/svelte', 'lucide-svelte'],
    },
  };
});
