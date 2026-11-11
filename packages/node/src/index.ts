import { GitVaultError } from './errors.js'
import { runCli } from './run-cli.js'

export { GitVaultError, type GitVaultErrorCode } from './errors.js'

export interface ConfigOptions {
  /** Vault id. Defaults to GITVAULT_VAULT, then the vault selected with `gitvault vault use`. */
  vault?: string
  /** Replace variables that are already set. Default false (like dotenv). */
  override?: boolean
  /** Path of the gitvault binary. Defaults to GITVAULT_BIN, then `gitvault` on PATH. */
  bin?: string
}

export interface ConfigResult {
  environment: string
  /** Names of the variables that were set. Values are never returned. */
  loaded: string[]
}

/**
 * Loads one environment's secrets into process.env. Synchronous, so it can run on the first line
 * of an app. It asks the installed `gitvault` CLI, which signs in through the GitHub CLI (or
 * GITVAULT_TOKEN on servers), checks the vault access and decrypts. Values are never logged and
 * never written to disk.
 *
 * @param environment defaults to GITVAULT_ENV, then NODE_ENV
 */
export function config(environment?: string, options: ConfigOptions = {}): ConfigResult {
  const name = environment ?? process.env.GITVAULT_ENV ?? process.env.NODE_ENV
  if (!name) {
    throw new GitVaultError(
      'E_USAGE',
      'no environment given',
      'pass one, or set GITVAULT_ENV or NODE_ENV',
    )
  }
  const bin = options.bin ?? process.env.GITVAULT_BIN ?? 'gitvault'
  const body = runCli(bin, exportArgs(name, options.vault ?? process.env.GITVAULT_VAULT))
  return { environment: name, loaded: apply(body.variables ?? {}, options.override === true) }
}

function exportArgs(name: string, vault: string | undefined): string[] {
  const args = ['--json', '--non-interactive']
  if (vault) args.push('--vault', vault)
  return [...args, 'env', 'export', name]
}

function apply(variables: Record<string, string>, override: boolean): string[] {
  const loaded: string[] = []
  for (const [key, value] of Object.entries(variables)) {
    if (override || process.env[key] === undefined) {
      process.env[key] = value
      loaded.push(key)
    }
  }
  return loaded
}
