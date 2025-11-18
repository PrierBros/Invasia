import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';

// https://astro.build/config
export default defineConfig({
  site: 'https://jprier.github.io',
  base: '/Invasia',
  integrations: [svelte()],
  vite: {
    optimizeDeps: {
      exclude: ['@astrojs/svelte']
    },
    server: {
      fs: {
        // Allow serving files from the wasm directory
        allow: ['..']
      }
    }
  }
});
