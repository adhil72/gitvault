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

export declare class GitVaultError extends Error {
  /** E_LOCKED, E_PERMISSION_DENIED, E_NOT_AUTHENTICATED, E_NOT_INSTALLED, E_NOT_FOUND, ... */
  code: string
  hint: string | null
}

/** Loads one environment's secrets into process.env (synchronous). */
export declare function config(environment?: string, options?: ConfigOptions): ConfigResult
