import { defineConfig } from 'tsup'

// Dual package: `require('gitvault')` and `import 'gitvault'` both work, with types.
export default defineConfig({
  entry: ['src/index.ts', 'src/config.ts', 'src/vite.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  clean: true,
  target: 'node18',
  outDir: 'dist',
})
