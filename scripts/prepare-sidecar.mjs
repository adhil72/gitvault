// Builds the `gitvault` CLI and copies it where Tauri expects a sidecar:
// apps/desktop/src-tauri/binaries/gitvault-<target-triple>[.exe]. The desktop installers then
// carry the CLI, so installing the app also installs `gitvault`.
// Usage: node scripts/prepare-sidecar.mjs [--target <triple>]
import { execFileSync } from 'node:child_process'
import { copyFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const flag = process.argv.indexOf('--target')
const requested = flag > -1 ? process.argv[flag + 1] : undefined

const hostTriple = () => {
  const info = execFileSync('rustc', ['-vV'], { encoding: 'utf8' })
  const host = /^host: (.+)$/m.exec(info)?.[1]
  if (!host) throw new Error('could not read the host target from `rustc -vV`')
  return host
}

const triple = requested ?? hostTriple()
const exe = triple.includes('windows') ? '.exe' : ''
const cargoArgs = ['build', '--release', '-p', 'gitvault-cli']
if (requested) cargoArgs.push('--target', requested)
execFileSync('cargo', cargoArgs, { cwd: root, stdio: 'inherit' })

const built = join(root, 'target', ...(requested ? [requested] : []), 'release', `gitvault${exe}`)
const outDir = join(root, 'apps/desktop/src-tauri/binaries')
mkdirSync(outDir, { recursive: true })
const dest = join(outDir, `gitvault-${triple}${exe}`)
copyFileSync(built, dest)
console.log(`sidecar ready: ${dest}`)
