import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  publicDir: '.generated/public',
  resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)), '@generated': fileURLToPath(new URL('./.generated', import.meta.url)) } },
  build: {
    outDir: 'dist', emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('/node_modules/')) return 'ui-vendor'
          if (id.endsWith('/.generated/data/cases.json')) return 'legal-fixtures'
        },
      },
    },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    include: ['tests/**/*.test.ts', 'tests/**/*.test.tsx'],
  },
})
