import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    outDir: "dist",
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        new_queue: resolve(__dirname, 'new_queue.html')
      }
    }
  },
  resolve: {
    alias: [
      { find: 'icons', replacement: '/src-tauri/icons' },
      { find: '@', replacement: '/src' },
      { find: 'react', replacement: 'preact/compat' },
      { find: 'react-dom/test-utils', replacement: 'preact/test-utils' },
      { find: 'react-dom', replacement: 'preact/compat' },
      { find: 'react/jsx-runtime', replacement: 'preact/jsx-runtime' }
    ],
  },
  plugins: [preact()]
})
