// After running `just setup-frontend`, merge this into the generated vite.config.js:
//
// import { defineConfig } from 'vite'
// import vue from '@vitejs/plugin-vue'
//
// export default defineConfig({
//   plugins: [vue()],
//   server: {
//     proxy: {
//       '/api': 'http://localhost:3000',   // ← add this
//     },
//   },
//   build: {
//     outDir: 'dist',                      // ← add this (usually already default)
//     emptyOutDir: true,                   // ← add this
//   },
// })
//
// Then run: just dashboard
// Then run: cargo build   (embeds dist/ into binary)
