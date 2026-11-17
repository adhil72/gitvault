# GitVault

Desktop and command-line secrets manager that stores **encrypted vaults in your own private GitHub repository**.
Sign in with GitHub, protect each vault with a master password, and use the same features from the app or the terminal.

- **Architecture and decisions:** [docs/architecture.md](docs/architecture.md) (section 16 lists what differs from the original plan)
- **Vault file format:** [docs/vault-format.md](docs/vault-format.md)
- **CLI reference (generated):** [docs/cli.md](docs/cli.md) · for AI tools: [llms.txt](llms.txt), [AGENTS.md](AGENTS.md)

## Install

On a server or desktop (Debian/Ubuntu via the signed apt repo, other Linux via cargo):

```bash
curl -fsSL https://raw.githubusercontent.com/adhil72/gitvault/main/scripts/install.sh | sh
```

This installs `gitvault` and its short alias **`gv`**. Sign in once with the GitHub CLI (`gh auth login`); GitVault reuses it, and does nothing if you are already signed in.

## Download

Windows (`.exe`) and Ubuntu (`.deb`) installers are published at https://adhil72.github.io/gitvault-website/download/. The installers also install the `gitvault` command-line tool (on `PATH`), so there is nothing else to set up. The Windows app updates itself; every release lists `SHA256SUMS` for verification. Ubuntu/Debian users can also install and update through a GPG-signed apt repository: see [docs/apt-repository.md](docs/apt-repository.md). Signing key fingerprint: `2F22 86FD 69D5 C056 80FC 12BF BCCE 9E57 C812 62F0`.

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

## How it's encrypted

Your vault is a private GitHub repository that contains **only ciphertext**. GitHub, and anyone who gets read access to the repo, never sees a secret value, name or note.

| Step                  | What GitVault uses                                                                                                   |
| --------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Master password → key | **Argon2id** (64 MiB memory, 3 iterations, random 16-byte salt) derives a key-encryption key                         |
| Vault key             | 32 random bytes, generated once per vault, stored only **wrapped** (encrypted) by your password and recovery key     |
| Each secret           | **AES-256-GCM** with a fresh 96-bit nonce on every write, one file per secret                                        |
| Recovery key          | 160 random bits shown once, run through HKDF-SHA256, so a forgotten password does not lock you out                   |
| Tamper protection     | The secret's vault id and file id are authenticated data, so a file copied to another vault or renamed fails to open |

What this means in practice:

- Changing your master password re-wraps the vault key; no secret has to be re-encrypted.
- Secret names, tags and notes are inside the ciphertext. The repo reveals only how many secrets exist and when they changed.
- Commit messages contain ids only, never names.
- The vault name and description are stored in plaintext in the header, so don't put secrets in them.
- Unlocked keys are kept only for the idle timeout (default 5 minutes), and `run`/`inject` never print secret values.
- There is no GitVault server. Your secrets are only ever sent, as ciphertext, to your own GitHub repository.

The full byte-level specification, including the file layout and key hierarchy, is in [docs/vault-format.md](docs/vault-format.md). Known limits are listed under [Security notes](#security-notes).

## How it compares

GitVault is built for **individual developers and very small teams** who want encrypted secrets they fully own, without running or paying for a server. It is not a replacement for a team platform with roles and audit trails.

|                           | `.env` files |       GitVault       |    1Password CLI    |      Doppler      |         Infisical          |           SOPS            |         git-crypt         |
| ------------------------- | :----------: | :------------------: | :-----------------: | :---------------: | :------------------------: | :-----------------------: | :-----------------------: |
| Encrypted at rest         |      No      |         Yes          |         Yes         |        Yes        |            Yes             |            Yes            |            Yes            |
| Where secrets live        |  Your disk   | Your own GitHub repo | 1Password's servers | Doppler's servers | Their cloud or your server |  Files in your git repo   |  Files in your git repo   |
| Account or server to run  |     None     |     GitHub only      |  1Password account  |  Doppler account  |   Account, or self-host    | None (needs a key source) | None (needs GPG or a key) |
| Open source               |      –       |         Yes          |         No          |        No         |            Yes             |            Yes            |            Yes            |
| Desktop app               |      –       |         Yes          |         Yes         |   Web dashboard   |       Web dashboard        |            No             |            No             |
| CLI `run` / inject        |      –       |         Yes          |         Yes         |        Yes        |            Yes             |            Yes            |            No             |
| Secret names hidden       |      No      |         Yes          |         Yes         |        Yes        |            Yes             |     Keys are visible      |    File names visible     |
| Version history           |      No      |    Yes (via git)     |         Yes         |        Yes        |            Yes             |       Yes (via git)       |       Yes (via git)       |
| MCP server for AI tools   |      –       |         Yes          |    Not built in     |   Not built in    |        Not built in        |            No             |            No             |
| Per-user roles and access |      –       |          No          |         Yes         |        Yes        |            Yes             |      Via key access       |      Via key access       |
| Team audit and rotation   |      –       |   Basic audit log    |         Yes         |        Yes        |            Yes             |            No             |            No             |
| Integrations (CI, cloud)  |      –       |    CLI and `.env`    |        Many         |       Many        |            Many            |        KMS and CI         |            No             |

### When to choose something else

- **You need per-person permissions, SSO or compliance reports:** use Doppler, Infisical or 1Password.
- **You want secrets committed next to your infrastructure code (GitOps):** SOPS fits that workflow better.
- **You only need to protect a few files in an existing repo:** git-crypt is simpler.

### When GitVault fits

- You want to own your secrets without running a service or paying per seat.
- You work solo or share a vault with one or two people you trust with the master password.
- You want the same vault from a desktop app, the terminal and AI tools, with secrets never printed.

Competitor features change; check each project's current documentation before deciding.

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

- **GitHub CLI** (no setup): run `gh auth login` once, then press "Continue with GitHub CLI" in the app or run `gv auth login` (it uses `gh` and does nothing if you are signed in).
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

For Vite projects, add the plugin instead (`import { gitvault } from '@adhil28/gitvault/vite'`, then
`plugins: [gitvault()]`). The Vite mode picks the environment, and only `VITE_` variables reach client code.

### CLI in 60 seconds

```bash
gh auth login                                   # once; gv signs in through it by itself
gv vault create --name Personal                 # prompts for a master password, prints the recovery key
echo -n "sk_live_…" | gv secret add STRIPE_KEY --environment production --value-stdin
gv secret list
cd ~/projects/shop && gv trust                  # asks which vault and environments, then trusts the folder
gv run --environment production -- ./deploy.sh  # secrets injected, never printed
gv sync now
```

Data lives in `~/.local/share/gitvault` (override with `GITVAULT_HOME`). Unlocked keys are kept for the idle timeout
(default 5 minutes); `gitvault unlock --export` prints a `GITVAULT_SESSION` token for scripts.

## Development

```bash
pnpm check:all     # frontend format/lint/types/tests + Rust fmt/clippy/tests
pnpm gen:types     # regenerate TypeScript types from the Rust DTOs
pnpm gen:docs      # regenerate docs/cli.md
pnpm bundle        # build the CLI sidecar, then the desktop installers that include it
```

## Security notes

- Secrets, names and notes are encrypted before they touch disk or GitHub; the repository only holds
  ciphertext plus a header with key-derivation settings. See [docs/vault-format.md](docs/vault-format.md).
- The GitHub token and unlocked-session keys are stored in a private (mode 0600) file, not yet in the OS keychain.
- The sign-in uses the `repo` scope (needed to create and push to a private repository). A narrower GitHub App is a
  recommended follow-up.
- Commits are unsigned; the app never shows a "Verified" badge.

## Contributing and license

See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

GitVault is released under the [MIT License](LICENSE).
