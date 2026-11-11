/** Stable error codes emitted by the CLI, plus the two this library adds itself. */
export type GitVaultErrorCode =
  | 'E_LOCKED'
  | 'E_PERMISSION_DENIED'
  | 'E_NOT_AUTHENTICATED'
  | 'E_NOT_FOUND'
  | 'E_NOT_INSTALLED'
  | 'E_USAGE'
  | 'E_INTERNAL'
  | (string & {})

export class GitVaultError extends Error {
  readonly code: GitVaultErrorCode
  readonly hint: string | null

  constructor(code: GitVaultErrorCode, message: string, hint?: string | null) {
    super(hint ? `${message} (${hint})` : message)
    this.name = 'GitVaultError'
    this.code = code
    this.hint = hint ?? null
  }
}
