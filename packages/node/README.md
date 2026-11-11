# gitvault (Node.js)

Written in TypeScript and published as a dual ESM + CommonJS package with bundled type definitions.

Load your GitVault secrets into `process.env` at startup. No `.env` files in the project.

## Install

Published on GitHub Packages. Point the scope at it and sign in with a token that has `read:packages` (and access to the repo):

```ini
# .npmrc
@adhil72:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

```bash
npm install @adhil72/gitvault
```

```js
import '@adhil72/gitvault/config' // uses GITVAULT_ENV, then NODE_ENV
```

```js
import { config } from '@adhil72/gitvault'
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
pnpm --filter @adhil72/gitvault build      # tsup -> dist (ESM, CJS, .d.ts)
pnpm --filter @adhil72/gitvault typecheck
pnpm test:node                    # builds the CLI and runs the integration tests
```
