import { config, type ConfigOptions } from './index.js'

export { GitVaultError, type GitVaultErrorCode } from './errors.js'

export interface GitVaultViteOptions extends ConfigOptions {
  /**
   * GitVault environment to load. Defaults to GITVAULT_ENV, then the Vite mode
   * (`development` for `vite`, `production` for `vite build`, or whatever `--mode` says).
   */
  environment?: string
}

// Structural types, so the plugin needs no runtime or type dependency on a particular Vite major.
interface ViteConfigEnv {
  mode: string
}

interface ViteResolvedConfig {
  logger: { info(message: string): void }
}

export interface GitVaultVitePlugin {
  name: string
  enforce: 'pre'
  config(userConfig: unknown, env: ViteConfigEnv): void
  configResolved(resolved: ViteResolvedConfig): void
}

/**
 * Vite plugin that loads one GitVault environment into `process.env` before Vite reads its env,
 * so no `.env` file is needed. Vite then exposes only the variables that start with `envPrefix`
 * (`VITE_` by default) to client code as `import.meta.env`. Everything else stays server-side
 * (the dev server, `vite.config.ts` plugins, SSR), exactly as it does with `.env` files.
 *
 * Variables already set in the shell win over the vault unless `override: true`, and the vault
 * wins over `.env` files. Values are never logged or written to disk.
 */
export function gitvault(options: GitVaultViteOptions = {}): GitVaultVitePlugin {
  const { environment, ...loaderOptions } = options
  let summary = ''

  return {
    name: 'gitvault',
    // Before other plugins' `config` hooks, so they can see the variables too.
    enforce: 'pre',
    config(_userConfig, env) {
      const name = environment ?? process.env.GITVAULT_ENV ?? env.mode
      const result = config(name, loaderOptions)
      summary = `loaded ${result.loaded.length} variable(s) from the "${result.environment}" environment`
    },
    configResolved(resolved) {
      // Counts only: names and values stay out of the log.
      resolved.logger.info(`gitvault: ${summary}`)
    },
  }
}

export default gitvault
