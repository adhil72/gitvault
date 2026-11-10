// Rebuilds apps/desktop/src/types/index.ts from the files ts-rs generated.
import { readdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..', 'apps', 'desktop', 'src', 'types')
const names = readdirSync(join(root, 'generated'))
  .filter((file) => file.endsWith('.ts'))
  .map((file) => file.slice(0, -3))
  .sort()

const lines = [
  '// Re-exports of the types generated from the Rust DTOs. Do not edit: run `pnpm gen:types`.',
  ...names.map((name) => `export type { ${name} } from './generated/${name}'`),
]
writeFileSync(join(root, 'index.ts'), `${lines.join('\n')}\n`)
console.log(`wrote ${names.length} type exports`)
