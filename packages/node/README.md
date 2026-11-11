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

## Development

```bash
pnpm --filter @adhil28/gitvault build      # tsup -> dist (ESM, CJS, .d.ts)
pnpm --filter @adhil28/gitvault typecheck
pnpm test:node                    # builds the CLI and runs the integration tests
```
