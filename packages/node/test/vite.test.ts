import test from 'node:test'
import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { resolveConfig, type Plugin } from 'vite'

const here = path.dirname(fileURLToPath(import.meta.url))

type PluginModule = typeof import('../src/vite.ts')

const BIN = path.resolve(here, '../../../target/debug/gitvault')
const PASSWORD = 'correct horse battery'
const root = fs.mkdtempSync(path.join(os.tmpdir(), 'gv-vite-root-'))
process.env.GITVAULT_HOME = fs.mkdtempSync(path.join(os.tmpdir(), 'gv-vite-'))
process.env.GITVAULT_BIN = BIN
process.env.GITVAULT_PASSWORD = PASSWORD
delete process.env.GITVAULT_SESSION
delete process.env.GITVAULT_VAULT
delete process.env.GITVAULT_ENV

function cli(args: string[], input = ''): void {
  execFileSync(BIN, args, { input, env: process.env, encoding: 'utf8' })
}

function addSecret(name: string, environment: string, value: string): void {
  cli(['secret', 'add', name, '--environment', environment, '--value-stdin'], value)
}

test.before(() => {
  assert.ok(fs.existsSync(BIN), 'build the CLI first: cargo build -p gitvault-cli')
  cli(
    ['vault', 'create', '--name', 'Demo', '--local-only', '--password-stdin', '--json'],
    `${PASSWORD}\n`,
  )
  addSecret('VITE_API_URL', 'development', 'http://localhost:9000')
  addSecret('VITE_API_URL', 'production', 'https://api.example.com')
  addSecret('SERVER_ONLY', 'development', 'dev-server-secret')
})

test.beforeEach(() => {
  for (const key of ['VITE_API_URL', 'SERVER_ONLY', 'GITVAULT_ENV']) delete process.env[key]
})

async function plugin(options?: Parameters<PluginModule['gitvault']>[0]): Promise<Plugin> {
  // A computed path, so type-checking does not depend on the build output existing.
  const built = path.join(here, '../dist/vite.js')
  const { gitvault } = (await import(built)) as PluginModule
  // The plugin is structurally typed on purpose, so Vite sees it as a plain Plugin.
  return gitvault(options) as unknown as Plugin
}

async function resolve(mode: string, options?: Parameters<PluginModule['gitvault']>[0]) {
  return resolveConfig(
    { root, configFile: false, logLevel: 'silent', plugins: [await plugin(options)] },
    mode === 'development' ? 'serve' : 'build',
    mode,
  )
}

test('the Vite mode picks the environment and VITE_ variables reach import.meta.env', async () => {
  const dev = await resolve('development')
  assert.equal(dev.env.VITE_API_URL, 'http://localhost:9000')

  delete process.env.VITE_API_URL
  const prod = await resolve('production')
  assert.equal(prod.env.VITE_API_URL, 'https://api.example.com')
})

test('variables without the env prefix stay out of import.meta.env but load server-side', async () => {
  const dev = await resolve('development')
  assert.equal(dev.env.SERVER_ONLY, undefined)
  assert.equal(process.env.SERVER_ONLY, 'dev-server-secret')
})

test('an explicit environment and GITVAULT_ENV both beat the mode', async () => {
  const explicit = await resolve('production', { environment: 'development' })
  assert.equal(explicit.env.VITE_API_URL, 'http://localhost:9000')

  delete process.env.VITE_API_URL
  process.env.GITVAULT_ENV = 'development'
  const fromEnv = await resolve('production')
  assert.equal(fromEnv.env.VITE_API_URL, 'http://localhost:9000')
})

test('a variable already set in the shell wins over the vault', async () => {
  process.env.VITE_API_URL = 'from-shell'
  const dev = await resolve('development')
  assert.equal(dev.env.VITE_API_URL, 'from-shell')
})

test('a missing CLI fails the config step with E_NOT_INSTALLED', async () => {
  await assert.rejects(
    resolve('development', { bin: '/no/such/gitvault' }),
    (e: { code?: string }) => e.code === 'E_NOT_INSTALLED',
  )
})
