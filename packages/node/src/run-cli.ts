import { spawnSync } from 'node:child_process'
import { GitVaultError } from './errors.js'

interface CliFailure {
  code?: string
  message?: string
  hint?: string
}

interface CliBody {
  error?: CliFailure
  variables?: Record<string, string>
}

/** Runs the gitvault CLI once and returns its parsed JSON body, or throws a GitVaultError. */
export function runCli(bin: string, args: string[]): CliBody {
  const result = spawnSync(bin, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    maxBuffer: 16 * 1024 * 1024,
  })
  // Only the CLI needs these: do not leave them in this process either.
  delete process.env.GITVAULT_PASSWORD
  delete process.env.GITVAULT_DEVICE_KEY

  if (result.error) throw spawnError(result.error)
  const body = parseBody(result.stdout, result.stderr)
  if (result.status !== 0 || body.error) throw cliError(body.error)
  return body
}

function spawnError(error: NodeJS.ErrnoException): GitVaultError {
  if (error.code === 'ENOENT') {
    return new GitVaultError(
      'E_NOT_INSTALLED',
      'the gitvault CLI was not found',
      'install GitVault, or set GITVAULT_BIN to its path',
    )
  }
  return new GitVaultError('E_INTERNAL', error.message)
}

function parseBody(stdout: string, stderr: string): CliBody {
  try {
    return JSON.parse(stdout) as CliBody
  } catch {
    throw new GitVaultError('E_INTERNAL', (stderr || 'unreadable output from gitvault').trim())
  }
}

function cliError(failure: CliFailure | undefined): GitVaultError {
  return new GitVaultError(
    failure?.code ?? 'E_INTERNAL',
    failure?.message ?? 'gitvault failed',
    failure?.hint,
  )
}
