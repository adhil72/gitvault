/** An error returned by a backend command (stable `code`, human `message`, optional `hint`). */
export class ApiError extends Error {
  readonly code: string
  readonly hint: string | null

  constructor(code: string, message: string, hint: string | null = null) {
    super(message)
    this.name = 'ApiError'
    this.code = code
    this.hint = hint
  }
}
