# GitVault

Desktop and command-line secrets manager that stores **encrypted vaults in your own private GitHub repository**.
Sign in with GitHub, protect each vault with a master password, and use the same features from the app or the terminal.

- **Architecture and decisions:** [docs/architecture.md](docs/architecture.md) (section 16 lists what differs from the original plan)
- **Vault file format:** [docs/vault-format.md](docs/vault-format.md)
- **CLI reference (generated):** [docs/cli.md](docs/cli.md) · for AI tools: [llms.txt](llms.txt), [AGENTS.md](AGENTS.md)

## What works

|                                                                          | Desktop app | CLI |
| ------------------------------------------------------------------------ | :---------: | :-: |
| Sign in with GitHub (device flow or personal access token)               |     ✅      | ✅  |
| Create / connect / unlock / lock vaults, recovery key, change password   |     ✅      | ✅  |
| Add, edit, rotate, delete secrets; environments, tags, favorites, expiry |     ✅      | ✅  |
| Version history and restore (from git)                                   |     ✅      | ✅  |
| Import / export `.env`                                                   |     ✅      | ✅  |
| Password / passphrase / PIN generator                                    |     ✅      | ✅  |
| Sync with GitHub, conflict resolution                                    |     ✅      | ✅  |
| Audit log, settings                                                      |     ✅      | ✅  |
| `run` / `inject` (secrets never printed), `--json`, schemas, completions |      –      | ✅  |
| MCP server for AI tools (`gitvault mcp`)                                 |      –      | ✅  |

Both front doors call the **same services** in `crates/gitvault-core`.

## Repository layout

```
crates/
  gitvault-crypto/   Argon2id, AES-256-GCM, key wrapping, recovery key (no I/O)
  gitvault-core/     domain, services, ports, DTOs                (all business logic)
  gitvault-infra/    adapters: files, git CLI, GitHub API, composition root
  gitvault-cli/      the `gitvault` binary
apps/desktop/
  src-tauri/         Tauri shell: thin commands over the services
  src/               React + Vite + TanStack Router + shadcn/ui
docs/                architecture, vault format, generated CLI reference
```

## Prerequisites

| Tool                           | Why                                                         |
| ------------------------------ | ----------------------------------------------------------- |
| Node 22+ and pnpm              | frontend                                                    |
| Rust (stable)                  | core, CLI, Tauri                                            |
| `git` on PATH                  | vault history and sync                                      |
| Tauri system libraries (Linux) | desktop build: https://tauri.app/start/prerequisites/#linux |

## Getting started

```bash
pnpm install
pnpm dev                       # desktop app (needs Rust + system libraries)
pnpm web                       # UI in a browser with a mock backend (press "Continue with GitHub CLI" for sample data)
cargo run -p gitvault-cli -- --help
```

### Signing in to GitHub

- **GitHub CLI** (no setup): run `gh auth login` once, then press "Continue with GitHub CLI" in the app or run `gitvault auth login --from-gh`.
- **Personal access token**: create a token with the `repo` scope, then
  `echo ghp_… | gitvault auth login --with-token` (or `gitvault auth login --from-gh` to reuse the GitHub CLI),
  or paste it under "sign in with a personal access token" in the app.
- **Browser sign-in (device flow):** create an OAuth app at https://github.com/settings/developers, enable
  _Device Flow_, then set its client id in Settings → General (or `GITVAULT_GITHUB_CLIENT_ID`).

### Use secrets in a Node.js app (no `.env` file)

```js
import 'gitvault/config' // loads GITVAULT_ENV (or NODE_ENV) into process.env, first line of your app
// or explicitly:
import { config } from 'gitvault'
config('production')
```

`config()` asks the installed `gitvault` CLI, so it uses the same GitHub sign-in and vault access check as
the app. Developers just sign in once (`gh auth login`, then unlock the vault). Servers and CI set two
variables instead: `GITVAULT_TOKEN` (a read-only GitHub token for the vault repo) and `GITVAULT_DEVICE_KEY`
(a per-server key from `gitvault device trust --name prod-server --export`, revocable with
`gitvault device revoke <id>`; `GITVAULT_PASSWORD` also works). Both are removed from the process after use. A locked vault without a password throws
`E_LOCKED`; it never prompts. Keep production in its own vault so only the right people have its password.
See [packages/node](packages/node).

### CLI in 60 seconds

```bash
gitvault auth login --from-gh
gitvault vault create --name Personal           # prompts for a master password, prints the recovery key
echo -n "sk_live_…" | gitvault secret add STRIPE_KEY --environment production --value-stdin
gitvault secret list
gitvault run --environment production -- ./deploy.sh      # secrets injected, never printed
gitvault sync now
```

Data lives in `~/.local/share/gitvault` (override with `GITVAULT_HOME`). Unlocked keys are kept for the idle timeout
(default 5 minutes); `gitvault unlock --export` prints a `GITVAULT_SESSION` token for scripts.

## Development

```bash
pnpm check:all     # frontend format/lint/types/tests + Rust fmt/clippy/tests
pnpm gen:types     # regenerate TypeScript types from the Rust DTOs
pnpm gen:docs      # regenerate docs/cli.md
```

## Security notes

- Secrets, names and notes are encrypted before they touch disk or GitHub; the repository only holds
  ciphertext plus a header with key-derivation settings. See [docs/vault-format.md](docs/vault-format.md).
- The GitHub token and unlocked-session keys are stored in a private (mode 0600) file, not yet in the OS keychain.
- The sign-in uses the `repo` scope (needed to create and push to a private repository). A narrower GitHub App is a
  recommended follow-up.
- Commits are unsigned; the app never shows a "Verified" badge.
