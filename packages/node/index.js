'use strict'

const { spawnSync } = require('node:child_process')

/** Error with a stable `code` (E_LOCKED, E_PERMISSION_DENIED, E_NOT_AUTHENTICATED, E_NOT_INSTALLED, ...). */
class GitVaultError extends Error {
  constructor(code, message, hint) {
    super(hint ? `${message} (${hint})` : message)
    this.name = 'GitVaultError'
    this.code = code
    this.hint = hint ?? null
  }
}

/**
 * Loads one environment's secrets into process.env. Synchronous, so it can run on the first line
 * of an app. It asks the installed `gitvault` CLI, which signs in through the GitHub CLI (or
 * GITVAULT_TOKEN on servers), checks the vault access and decrypts. Values are never logged and
 * never written to disk.
 *
 * @param {string} [environment] defaults to GITVAULT_ENV, then NODE_ENV
 * @param {{ vault?: string, override?: boolean, bin?: string }} [options]
 * @returns {{ environment: string, loaded: string[] }} names only, never values
 */
function config(environment, options = {}) {
  const name = environment ?? process.env.GITVAULT_ENV ?? process.env.NODE_ENV
  if (!name) {
    throw new GitVaultError(
      'E_USAGE',
      'no environment given',
      'pass one, or set GITVAULT_ENV or NODE_ENV',
    )
  }
  const args = ['--json', '--non-interactive']
  const vault = options.vault ?? process.env.GITVAULT_VAULT
  if (vault) args.push('--vault', vault)
  args.push('env', 'export', name)

  const result = spawnSync(options.bin ?? process.env.GITVAULT_BIN ?? 'gitvault', args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    maxBuffer: 16 * 1024 * 1024,
  })
  // The password is only needed by the CLI: do not leave it in this process either.
  delete process.env.GITVAULT_PASSWORD
  delete process.env.GITVAULT_DEVICE_KEY

  if (result.error) {
    const missing = result.error.code === 'ENOENT'
    throw new GitVaultError(
      missing ? 'E_NOT_INSTALLED' : 'E_INTERNAL',
      missing ? 'the gitvault CLI was not found' : result.error.message,
      missing ? 'install GitVault, or set GITVAULT_BIN to its path' : undefined,
    )
  }
  return apply(parse(result), name, options.override === true)
}

function parse(result) {
  let body
  try {
    body = JSON.parse(result.stdout)
  } catch {
    throw new GitVaultError(
      'E_INTERNAL',
      (result.stderr || 'unreadable output from gitvault').trim(),
    )
  }
  if (result.status !== 0 || body.error) {
    const { code = 'E_INTERNAL', message = 'gitvault failed', hint } = body.error ?? {}
    throw new GitVaultError(code, message, hint)
  }
  return body
}

function apply(body, name, override) {
  const loaded = []
  for (const [key, value] of Object.entries(body.variables)) {
    if (override || process.env[key] === undefined) {
      process.env[key] = value
      loaded.push(key)
    }
  }
  return { environment: name, loaded }
}

module.exports = { config, GitVaultError }
