# gitvault (Node.js)

Written in TypeScript and published as a dual ESM + CommonJS package with bundled type definitions.

Load your GitVault secrets into `process.env` at startup. No `.env` files in the project.

## Install

Published on the public npm registry. No token needed:

```bash
npm install @adhil28/gitvault
```

```js
import '@adhil28/gitvault/config' // uses GITVAULT_ENV, then NODE_ENV
```

```js
import { config } from '@adhil28/gitvault'
const { environment, loaded } = config('production') // names only, never values
```

- Synchronous, so it works on the first line of an app. Existing variables win unless `{ override: true }`.
- Secrets of the environment plus shared ones (no environment) are loaded; the environment's own value wins.
- Requires the `gitvault` CLI on PATH (or `GITVAULT_BIN`).
- Developers: sign in once with `gh auth login`, then trust the device once (`gitvault device trust --name laptop`, or the checkbox on the app's unlock page). After that it unlocks silently, with no password and no `.env`. To limit it to a project, run `gitvault trust add . --env development` inside the project instead: the device then unlocks by itself only in that folder and its subfolders (and only for the listed environments). Elsewhere, `config()` throws `E_LOCKED`.
- Servers/CI: `gitvault device trust --name prod-server --export` prints a key once. Set it as `GITVAULT_DEVICE_KEY` together with `GITVAULT_TOKEN` (read-only GitHub token for the vault repo). Revoke it any time with `gitvault device revoke <id>`. `GITVAULT_PASSWORD` also works.
- Errors are `GitVaultError` with a `code`: `E_LOCKED`, `E_PERMISSION_DENIED`, `E_NOT_AUTHENTICATED`, `E_NOT_INSTALLED`, ...

## Vite plugin

Load an environment into a Vite project without a `.env` file:

```ts
// vite.config.ts
import { defineConfig } from 'vite'
import { gitvault } from '@adhil28/gitvault/vite'

export default defineConfig({
  plugins: [gitvault()], // or gitvault({ environment: 'staging', vault: '...' })
})
```

- The environment is `options.environment`, then `GITVAULT_ENV`, then the Vite mode (`vite` is `development`, `vite build` is `production`, `--mode staging` is `staging`).
- Vite exposes only variables that start with `envPrefix` (`VITE_` by default) to client code as `import.meta.env`. Every other secret loads into `process.env` for the dev server and build tooling, but never reaches the browser bundle. Name client-safe values `VITE_...` in the vault, and never put a real secret behind that prefix.
- Variables already set in the shell win over the vault (`override: true` flips that), and the vault wins over `.env` files.
- The plugin runs after `vite.config.ts` is evaluated, so the config file itself cannot read vault values from `process.env`. Use `import '@adhil28/gitvault/config'` at the top of the config file if it must.
- It logs only how many variables were loaded, never names or values. A missing CLI or a locked vault fails the start with a `GitVaultError`.
- Works with Vite 5 and later (`vite` is an optional peer dependency). The same options as `config()` apply: `vault`, `override`, `bin`.

## Development

```bash
pnpm --filter @adhil28/gitvault build      # tsup -> dist (ESM, CJS, .d.ts)
pnpm --filter @adhil28/gitvault typecheck
pnpm test:node                    # builds the CLI and runs the integration tests
```

## License

MIT. See [LICENSE](./LICENSE).
