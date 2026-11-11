import test from 'node:test'
import assert from 'node:assert/strict'
import { execFileSync, spawnSync } from 'node:child_process'
import { createRequire } from 'node:module'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const here = path.dirname(fileURLToPath(import.meta.url))
const require = createRequire(import.meta.url)

type Loader = typeof import('../src/index.ts')

const BIN = path.resolve(here, '../../../target/debug/gitvault')
const PASSWORD = 'correct horse battery'
const home = fs.mkdtempSync(path.join(os.tmpdir(), 'gv-node-'))
process.env.GITVAULT_HOME = home
process.env.GITVAULT_BIN = BIN
delete process.env.GITVAULT_SESSION
delete process.env.GITVAULT_VAULT

function cli(args: string[], input = ''): string {
  return execFileSync(BIN, args, { input, env: process.env, encoding: 'utf8' })
}

test.before(() => {
  assert.ok(fs.existsSync(BIN), 'build the CLI first: cargo build -p gitvault-cli')
  cli(
    ['vault', 'create', '--name', 'Demo', '--local-only', '--password-stdin', '--json'],
    `${PASSWORD}\n`,
  )
  cli(['secret', 'add', 'SHARED', '--value-stdin'], 'shared-value')
  cli(['secret', 'add', 'API_KEY', '--environment', 'development', '--value-stdin'], 'dev-key')
  cli(['secret', 'add', 'API_KEY', '--environment', 'production', '--value-stdin'], 'prod-key')
  cli(['secret', 'add', 'SHARED', '--environment', 'production', '--value-stdin'], 'prod-shared')
  cli(['lock'])
})

// The built CommonJS bundle, reloaded each time so every test starts from a clean module.
function freshLoader(): Loader {
  const file = path.join(here, '../dist/index.cjs')
  delete require.cache[file]
  return require(file) as Loader
}

test('a locked vault without a password throws E_LOCKED and never hangs', () => {
  const { config, GitVaultError } = freshLoader()
  assert.throws(
    () => config('development'),
    (e: unknown) => e instanceof GitVaultError && e.code === 'E_LOCKED',
  )
})

test('GITVAULT_PASSWORD unlocks, loads one environment, and is removed afterwards', () => {
  const { config } = freshLoader()
  process.env.GITVAULT_PASSWORD = PASSWORD
  delete process.env.API_KEY
  delete process.env.SHARED
  const result = config('development')
  assert.equal(process.env.API_KEY, 'dev-key')
  assert.equal(process.env.SHARED, 'shared-value')
  assert.deepEqual(result.loaded.sort(), ['API_KEY', 'SHARED'])
  assert.equal(process.env.GITVAULT_PASSWORD, undefined)
  assert.ok(!JSON.stringify(result).includes('dev-key'), 'values are never returned')
})

test('environment-specific values win over shared ones, and other environments stay out', () => {
  const { config } = freshLoader()
  delete process.env.API_KEY
  delete process.env.SHARED
  config('production')
  assert.equal(process.env.API_KEY, 'prod-key')
  assert.equal(process.env.SHARED, 'prod-shared')
})

test('existing variables are kept unless override is set', () => {
  const { config } = freshLoader()
  process.env.API_KEY = 'from-shell'
  config('development')
  assert.equal(process.env.API_KEY, 'from-shell')
  config('development', { override: true })
  assert.equal(process.env.API_KEY, 'dev-key')
})

test('a trusted device key unlocks without the password, and stops working once revoked', () => {
  const { config } = freshLoader()
  const exported = JSON.parse(
    cli(
      ['--json', 'device', 'trust', '--name', 'ci', '--export', '--password-stdin'],
      `${PASSWORD}\n`,
    ),
  )
  const key = exported.device_key
  cli(['lock'])
  delete process.env.API_KEY
  process.env.GITVAULT_DEVICE_KEY = key
  config('development')
  assert.equal(process.env.API_KEY, 'dev-key')
  assert.equal(process.env.GITVAULT_DEVICE_KEY, undefined, 'the key is removed after use')

  cli(['device', 'revoke', exported.device.id, '--password-stdin'], `${PASSWORD}\n`)
  cli(['lock'])
  process.env.GITVAULT_DEVICE_KEY = key
  assert.throws(
    () => config('development'),
    (e: { code?: string }) => e.code === 'E_PERMISSION_DENIED',
  )
})

test('a missing CLI gives E_NOT_INSTALLED', () => {
  const { config } = freshLoader()
  assert.throws(
    () => config('development', { bin: '/no/such/gitvault' }),
    (e: { code?: string }) => e.code === 'E_NOT_INSTALLED',
  )
})

function childEnv(): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    NODE_ENV: 'production',
    GITVAULT_PASSWORD: PASSWORD,
  }
  delete env.API_KEY
  return env
}

test('importing gitvault/config uses NODE_ENV (CommonJS)', () => {
  const run = spawnSync(
    process.execPath,
    ['-e', "require('./dist/config.cjs'); console.log(process.env.API_KEY)"],
    { cwd: path.join(here, '..'), env: childEnv(), encoding: 'utf8' },
  )
  assert.equal(run.stdout.trim(), 'prod-key')
})

test('importing gitvault/config uses NODE_ENV (ESM)', () => {
  const run = spawnSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      "await import('./dist/config.js'); console.log(process.env.API_KEY)",
    ],
    { cwd: path.join(here, '..'), env: childEnv(), encoding: 'utf8' },
  )
  assert.equal(run.stdout.trim(), 'prod-key')
})
