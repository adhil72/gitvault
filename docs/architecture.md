# GitVault — Architecture & Delivery Plan

> A desktop secrets manager whose vaults live as encrypted, versioned files in **your own private GitHub repository**.
> Built with **Tauri 2 (Rust)** + **React + Vite + TypeScript**. Every feature is available from the **terminal** through the same service layer the UI uses.

Status: **Draft v2 — for review before any code is written.**
Project folder: `/home/adhil/projects/gitvault`
Design source: Figma file "Vaultkeeper – Secrets Manager" (product name is **GitVault**; the Figma text has been renamed).

---

## 0. Summary of key decisions

| #   | Decision                                                                                                                                                                                               | Why                                                                                                              |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| D1  | **One Rust core library** (`gitvault-core`) holds all business logic. Desktop app and CLI are thin shells over it.                                                                                     | Guarantees UI/CLI parity. A feature is "done" only when the service exists; UI and CLI are just two front doors. |
| D2  | **Hexagonal (ports & adapters)** inside the core. Domain and use cases never touch the filesystem, network, git or keychain directly.                                                                  | Testable with in-memory fakes, swappable infrastructure, readable code.                                          |
| D3  | **Cargo workspace of small crates** with one-way dependencies, enforced by the compiler.                                                                                                               | Layer violations become compile errors, not code-review comments.                                                |
| D4  | **One encrypted file per secret** in the vault repo (`secrets/<uuid>.enc`).                                                                                                                            | Clean git diffs, per-secret history, rare merge conflicts, no metadata leaked in file names or commit messages.  |
| D5  | **Key hierarchy:** master password → Argon2id → KEK → unwraps random **vault key**; a **recovery key** can also unwrap it.                                                                             | Changing the master password re-wraps one key instead of re-encrypting every secret.                             |
| D6  | **GitHub login via OAuth Device Flow**; token stored in the **OS keychain**.                                                                                                                           | Works for a desktop app and a headless CLI, no client secret shipped.                                            |
| D7  | **Typed contracts end to end:** Rust DTOs are the single source of truth; TypeScript types and CLI JSON schemas are generated from them.                                                               | No drift between UI, CLI and core.                                                                               |
| D8  | **CLI is AI-agent friendly by design:** `--json` everywhere, stable exit codes, no prompts in non-TTY, machine-readable command catalog, JSON Schemas, `run`/`inject` so agents never see raw secrets. | The user's explicit requirement.                                                                                 |
| D9  | **Frontend: TanStack Router (file-based `routes/`) + `components/{ui,general,pages}` + shadcn/ui** with enforced import boundaries.                                                                    | Thin routes, reusable building blocks, pages composed from them.                                                 |
| D10 | **Strict, automated code rules** (lint, format, size limits, boundaries, CI gates).                                                                                                                    | Quality is enforced by tools, not memory.                                                                        |

> ⚠️ **Open decision that affects the design (see §14, R1):** the current Figma auth screen says GitVault requests only `read:user` and `user:email` and has "no access to your repositories". Creating a private vault repo and pushing commits **requires repository write access**. The UI copy must change.

---

## 1. Goals, non-goals, principles

### 1.1 Goals

1. Store API keys, passwords, certificates, notes and `.env` files, encrypted, in a GitHub repo the user owns.
2. Sign in **only with GitHub**. A master password protects each vault (set when a vault is created).
3. Full feature parity between desktop UI and CLI, through **one shared service layer**.
4. Usable by AI tools and scripts: predictable commands, JSON output, safe secret handling.
5. Readable, modular, well-tested code that a new contributor can navigate in minutes.
6. Works on macOS, Windows, Linux.

### 1.2 Non-goals (v1)

- Real-time multi-user collaboration or per-user permissions inside one vault.
- A hosted backend. There is **no GitVault server**. GitHub is storage and identity only.
- Mobile apps, browser extension, autofill.

### 1.3 Engineering principles

- **Security first, simplicity second, features third.**
- **One way to do each thing.** Logic lives in exactly one place (the core).
- **Make illegal states unrepresentable** (types, newtypes, enums) instead of validating everywhere.
- **Fail loudly and typed.** Errors are values with stable codes.
- **Secrets are radioactive:** minimal lifetime, never logged, zeroized, never in URLs/args/commit messages.
- **Boring technology, sharp boundaries.**

---

## 2. System overview

```
┌────────────────────────┐        ┌────────────────────────┐        ┌────────────────────────┐
│  Desktop UI (React)    │        │  CLI  `gitvault`       │        │  AI tools / scripts    │
│  Vite + TypeScript     │        │  clap                  │◄───────│  (shell, MCP later)    │
└───────────┬────────────┘        └───────────┬────────────┘        └────────────────────────┘
            │ Tauri IPC (typed commands)      │ in-process calls
┌───────────▼────────────┐                    │
│  src-tauri (Rust)      │                    │
│  thin command handlers │                    │
└───────────┬────────────┘                    │
            └──────────────────┬──────────────┘
                               │   same `Services` object (built by one bootstrap fn)
                  ┌────────────▼─────────────┐
                  │      gitvault-core       │   domain + use cases + port traits
                  │  (no I/O, no frameworks) │
                  └────────────┬─────────────┘
                               │ ports (traits)
                  ┌────────────▼─────────────┐
                  │      gitvault-infra      │   adapters
                  │ git · GitHub · keychain  │
                  │ filesystem · SQLite      │
                  └────────────┬─────────────┘
                               │
              ┌────────────────┴──────────────────┐
              │ GitHub (private repo)  │ OS keychain │ local app data │
              └───────────────────────────────────┘
```

**The parity rule:** UI commands and CLI commands may only _translate input/output_ and call a service method. If a handler contains business logic, it is a bug.

---

## 3. Tech stack

| Area               | Choice                                                                                                           | Notes                                                             |
| ------------------ | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Desktop shell      | **Tauri 2**                                                                                                      | Small binaries, Rust backend, permission-based capabilities.      |
| Core language      | **Rust (stable, edition 2021+)**                                                                                 | Memory safety, `zeroize`, strong typing.                          |
| Frontend           | **React 18 + Vite + TypeScript (strict)**                                                                        |                                                                   |
| Routing            | **TanStack Router** (file-based, Vite plugin, type-safe)                                                         | Hash history under Tauri.                                         |
| Server state       | **TanStack Query**                                                                                               | Wraps Tauri commands; caching, retries, invalidation.             |
| UI state           | **Zustand** (small, local)                                                                                       | Only for UI concerns (modals, wizard step, theme).                |
| Forms / validation | **react-hook-form + zod**                                                                                        |                                                                   |
| Styling            | **Tailwind CSS** + CSS variables from design tokens                                                              | Light/dark via `data-theme`.                                      |
| Primitives         | **shadcn/ui** (Radix + Tailwind) via the shadcn CLI, customized in `components/ui`                               | Accessible dialogs, menus, selects.                               |
| Icons              | lucide-react                                                                                                     | Matches the Figma icon set.                                       |
| Crypto             | RustCrypto: `aes-gcm`, `argon2`, `hkdf`, `rand`, `zeroize`, `secrecy`                                            | No custom crypto.                                                 |
| Git                | `git2` (libgit2) for local repo ops                                                                              | HTTPS + token auth.                                               |
| GitHub API         | `reqwest` + typed models                                                                                         | Device flow, repo discovery/creation.                             |
| Keychain           | `keyring` crate                                                                                                  | macOS Keychain, Windows Credential Manager, Linux Secret Service. |
| Local DB           | `rusqlite` (audit log, cache only)                                                                               | **Never** stores secret values.                                   |
| CLI                | `clap` (derive), `dialoguer` (prompts), `indicatif`                                                              |                                                                   |
| Errors             | `thiserror` (libraries), `anyhow` (binaries only)                                                                |                                                                   |
| Logging            | `tracing` with redaction                                                                                         |                                                                   |
| Type sharing       | `tauri-specta` (TS bindings) + `schemars` (JSON Schema)                                                          | Generated, never hand-written.                                    |
| Package mgmt       | **pnpm** workspaces + **Cargo** workspace                                                                        |                                                                   |
| Lint / format      | `clippy` (pedantic subset), `rustfmt`; **ESLint + Prettier** (or Biome)                                          |                                                                   |
| Tests              | `cargo-nextest`, `assert_cmd`, `insta`; **Vitest**, Testing Library, **Playwright**; `tauri-driver` for real e2e |                                                                   |
| Supply chain       | `cargo-deny`, `cargo-audit`, `pnpm audit`, Dependabot/Renovate                                                   |                                                                   |
| Releases           | `tauri-action`, `release-please`, signed updater                                                                 |                                                                   |

---

## 4. Repository layout

```
gitvault/
├─ Cargo.toml                    # Cargo workspace
├─ pnpm-workspace.yaml
├─ AGENTS.md                     # rules for AI coding tools (mirror of §10, short)
├─ README.md
├─ docs/
│  ├─ architecture.md            # this plan, kept current
│  ├─ adr/                       # architecture decision records (0001-….md)
│  ├─ vault-format.md            # on-disk/repo format spec (versioned)
│  ├─ cli.md                     # generated CLI reference
│  └─ security.md                # threat model
├─ crates/
│  ├─ gitvault-crypto/           # pure crypto: kdf, aead, key wrapping, zeroize (no I/O)
│  ├─ gitvault-core/             # domain + application services + ports (no I/O)
│  ├─ gitvault-infra/            # adapters implementing ports + composition root
│  └─ gitvault-cli/              # `gitvault` binary
├─ apps/
│  └─ desktop/
│     ├─ src-tauri/              # Tauri shell (Rust) – thin commands only
│     └─ src/                    # React frontend (routes/, components/{ui,general,pages}, … see §9.3)
├─ packages/
│  ├─ tokens/                    # design tokens (generated from Figma variables → CSS/TS)
│  └─ node/                      # `@adhil28/gitvault`: loads a vault into process.env, Vite plugin
├─ scripts/                      # dev scripts (codegen, release helpers)
└─ .github/
   ├─ workflows/                 # ci.yml, release.yml, security.yml
   └─ ISSUE_TEMPLATE/ , PULL_REQUEST_TEMPLATE.md
```

**Dependency direction (enforced by Cargo):**

```
gitvault-crypto  ◄──  gitvault-core  ◄──  gitvault-infra  ◄──  gitvault-cli
                                                      ▲
                                        apps/desktop/src-tauri
```

- `crypto` depends on nothing internal.
- `core` depends only on `crypto`. **No `tokio::fs`, `git2`, `reqwest`, `keyring`, `rusqlite` in core.**
- `infra` implements core's ports.
- `cli` and `src-tauri` depend on `core` + `infra` (for the bootstrap), never on each other.

### 4.1 `gitvault-core` internals

```
crates/gitvault-core/src/
├─ lib.rs
├─ error.rs                      # CoreError + stable ErrorCode enum
├─ domain/                       # pure types and rules (no async, no I/O)
│  ├─ vault/        (vault.rs, vault_id.rs, vault_header.rs)
│  ├─ secret/       (secret.rs, secret_kind.rs, secret_name.rs, secret_value.rs, version.rs)
│  ├─ organization/ (group.rs, environment.rs, tag.rs)
│  ├─ account/      (github_account.rs)
│  ├─ audit/        (audit_event.rs)
│  └─ settings/     (settings.rs)
├─ ports/                        # traits the core needs from the outside
│  ├─ vault_repository.rs        # read/write encrypted vault files
│  ├─ git_host.rs                # GitHub: auth, list/create repos
│  ├─ sync_transport.rs          # pull/push/commit history
│  ├─ key_store.rs               # OS keychain (tokens, session keys)
│  ├─ audit_log.rs
│  ├─ clock.rs, id_generator.rs  # make time and ids injectable
│  └─ clipboard.rs               # clear-after-timeout (desktop only impl)
├─ application/                  # use cases = the "services" the UI/CLI call
│  ├─ auth/        (login_with_github.rs, logout.rs, current_account.rs)
│  ├─ vaults/      (list_vaults.rs, create_vault.rs, open_vault.rs, unlock_vault.rs, lock_vault.rs, delete_vault.rs)
│  ├─ secrets/     (add_secret.rs, get_secret.rs, list_secrets.rs, update_secret.rs, rotate_secret.rs,
│  │                delete_secret.rs, restore_version.rs, secret_history.rs, search_secrets.rs)
│  ├─ organization/(manage_groups.rs, manage_environments.rs, manage_tags.rs)
│  ├─ transfer/    (import_env.rs, export_env.rs)
│  ├─ tools/       (generate_password.rs, run_with_secrets.rs)
│  ├─ sync/        (sync_now.rs, sync_status.rs, resolve_conflict.rs)
│  ├─ audit/       (list_audit_events.rs)
│  └─ settings/    (get_settings.rs, update_settings.rs)
├─ dto/                          # serde + specta + schemars types crossing the boundary
└─ services.rs                   # `Services` struct: one field per application module
```

**Rules for the core**

- One use case = one file = one struct with one public `execute` method.
- A use case receives its dependencies (ports) through its constructor as `Arc<dyn Trait>`.
- Use cases return **DTOs**, never domain internals (`SecretValue` never leaves except through an explicit `reveal`).
- Domain types validate in constructors (`SecretName::parse`, `VaultName::parse`), so invalid values cannot exist.

### 4.2 `gitvault-infra` internals

```
crates/gitvault-infra/src/
├─ bootstrap.rs                  # build_services(config) -> Services   ← the ONE composition root
├─ config.rs                     # paths, env vars, defaults
├─ git/            (git2_transport.rs, commit_builder.rs, merge.rs)
├─ github/         (device_flow.rs, api_client.rs, repo_discovery.rs, models.rs)
├─ keychain/       (os_key_store.rs, in_memory_key_store.rs)
├─ storage/        (fs_vault_repository.rs, vault_layout.rs)
├─ audit/          (sqlite_audit_log.rs, migrations/)
├─ system/         (system_clock.rs, uuid_generator.rs, clipboard.rs)
└─ testing/        (fakes used by core integration tests)   # behind `testing` feature
```

### 4.3 `gitvault-cli` internals

```
crates/gitvault-cli/src/
├─ main.rs                       # parse → bootstrap → dispatch → exit code
├─ cli.rs                        # clap definitions (global flags + subcommands)
├─ commands/                     # one file per command group, each fn ≤ 30 lines
│  ├─ auth.rs  vault.rs  secret.rs  group.rs  env.rs  tag.rs
│  ├─ import_export.rs  generate.rs  run.rs  sync.rs  audit.rs  settings.rs  schema.rs  catalog.rs
├─ output/                       # human tables vs `--json` rendering (one place)
│  ├─ renderer.rs  table.rs  json.rs  redact.rs
├─ prompt/                       # interactive prompts, auto-disabled when not a TTY
├─ session.rs                    # unlock/session handling for the CLI process
└─ exit_code.rs                  # ErrorCode → process exit code
```

---

## 5. Domain model

| Entity              | Key fields                                                                                                                                           | Notes                                                                                                                 |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| **Account**         | github_login, github_id, avatar_url                                                                                                                  | From GitHub; token stays in keychain.                                                                                 |
| **Vault**           | id (uuid), name, description, repo (`owner/name`), default branch, created_at                                                                        | One vault ⇒ one private GitHub repo.                                                                                  |
| **Secret**          | id (uuid), name, kind, group_id?, environment_id?, tags[], value, notes, expires_at?, remind_before_days?, favorite, created_at, updated_at, version | `kind` ∈ `api_key`, `password`, `certificate`, `note`, `env_file`.                                                    |
| **SecretVersion**   | secret_id, version, changed_by, device, changed_at, commit_sha, change_summary                                                                       | Derived from git history + encrypted change log.                                                                      |
| ~~Group~~ (removed) | id, name, color?                                                                                                                                     | **Replaces "Folder"** (Figma updated). A named collection of related secrets, e.g. _Stripe_, _Backend API_. Optional. |
| **Environment**     | id, name, color                                                                                                                                      | Defaults: Production, Staging, Development.                                                                           |
| **Tag**             | name                                                                                                                                                 | Free-form labels.                                                                                                     |
| **AuditEvent**      | id, at, kind, secret_ref?, device, commit_sha?                                                                                                       | `viewed`, `copied`, `created`, `updated`, `rotated`, `deleted`, `exported`, `synced`. Local only.                     |
| **Settings**        | auto_lock_minutes, clear_clipboard_seconds, hide_revealed_seconds, biometric_unlock, theme, auto_sync                                                | Stored locally (not in the vault repo).                                                                               |

Groups were removed: a **vault is the unit of access** (one GitHub repo, permissions managed on GitHub), so separate access needs separate vaults (e.g. `app-dev`, `app-prod`). Environment and tags remain as labels.

---

## 6. Security architecture

### 6.1 Key hierarchy

```
master password ──Argon2id(salt, params)──► KEK (32 bytes)
                                              │ AES-256-GCM unwrap
recovery key (random 160-bit, shown once) ─HKDF─► KEK_r
                                              │ AES-256-GCM unwrap
                                              ▼
                                    VAULT KEY (random 32 bytes, per vault)
                                              │ AES-256-GCM, fresh 96-bit nonce per write
                                              ▼
                          secret envelope  (value + name + metadata, all encrypted)
```

- `vault.json` stores **two wrapped copies** of the vault key (password-wrapped, recovery-wrapped) plus KDF parameters and format version. No plaintext secret data.
- **Changing the master password** = unwrap vault key, re-wrap with new KEK. No secret is re-encrypted.
- **Rotating the vault key** (rare, explicit) = re-encrypt every secret file, one commit.
- AEAD **associated data** binds ciphertext to `vault_id + secret_id + format_version` (prevents file swapping).
- Argon2id defaults: memory ≥ 64 MiB, iterations ≥ 3, parallelism 1–4; parameters stored in `vault.json` so they can be raised later.
- Everything secret in memory uses `secrecy::SecretBox` / `zeroize`. `gitvault-core` and `gitvault-crypto` use `#![forbid(unsafe_code)]`.

### 6.2 Vault repository format (versioned, documented in `docs/vault-format.md`)

```
<vault-repo>/
├─ .gitvault/
│  └─ vault.json              # { format: 1, vault_id, kdf: {…}, wrapped_key_password, wrapped_key_recovery }
├─ secrets/
│  ├─ 7f3c…e1.enc             # { v:1, nonce, ciphertext }  — one file per secret, uuid file names
│  └─ …
├─ index/                     # optional encrypted index for fast listing (rebuilt on conflict)
└─ README.md                  # human note: "Encrypted GitVault vault. Do not edit by hand."
```

- Repo topic **`gitvault-vault`** is added on creation so the app can **discover** existing vaults via the GitHub API.
- **Commit messages contain only ids** (`secret: update 7f3c`), never names or values.
- Repo is created **private**; the app refuses to sync to a public repo.

### 6.3 Unlock & sessions

| Context              | Mechanism                                                                                                                                                                                                      |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Desktop              | Vault key held in `SessionManager` (in-process, zeroized on lock/auto-lock/quit).                                                                                                                              |
| CLI                  | `gitvault unlock` creates a short-lived **session**: the vault key is sealed with a random session key kept in the OS keychain (TTL, default 15 min). Prints a `GITVAULT_SESSION` token for use in scripts/CI. |
| AI tools / CI        | Provide `GITVAULT_SESSION` (preferred) or run `gitvault unlock --password-stdin`. **No password on the command line, ever.**                                                                                   |
| Biometric (optional) | Stores the vault key in the OS secure store gated by Touch ID / Windows Hello.                                                                                                                                 |

### 6.4 Threat model (summary — full version in `docs/security.md`)

| Threat                                | Mitigation                                                                                                            |
| ------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| GitHub account/repo compromised       | Repo only holds ciphertext; master password never leaves the device; Argon2id slows brute force.                      |
| Malware reading the clipboard         | Clear-after-N-seconds done in Rust; copy only on explicit action.                                                     |
| XSS / malicious content in webview    | Strict CSP, no remote content, no `eval`, no `dangerouslySetInnerHTML`, minimal Tauri capabilities.                   |
| Secrets leaking into logs/crash dumps | `tracing` redaction layer, `Debug` for secret types prints `[REDACTED]`, no secrets in args/URLs/commit messages.     |
| Tampering with repo files             | AEAD authentication + associated data; unsigned/invalid files are rejected and reported.                              |
| Supply chain                          | `cargo-deny`, `cargo-audit`, `pnpm audit`, pinned lockfiles, signed releases, minimal dependencies.                   |
| Process memory scraping               | Short key lifetime, zeroize on lock, auto-lock timer. (Cannot fully defeat a privileged local attacker — documented.) |

### 6.5 Tauri hardening

- Tauri 2 **capabilities**: grant only the specific commands/events the UI needs; **no** `shell`, `fs`, or `http` plugins exposed to the webview.
- Strict `Content-Security-Policy`; webview loads only bundled assets.
- All IPC inputs validated in Rust (not trusted from the frontend).
- Auto-updater uses signed updates only.

---

## 7. GitHub integration & sync

### 7.1 Authentication (GitHub only)

1. **Device Flow:** app requests a device + user code, shows the code (matches Figma "Authorize device" screen), opens `github.com/login/device`, polls until authorized.
2. Token is stored in the **OS keychain** (`keyring`). Never written to disk, logs or the repo.
3. `gitvault auth login` performs the same flow in the terminal; desktop and CLI **share the keychain entry**, so logging in once covers both.
4. Prefer a **GitHub App** with fine-grained permissions (Contents: read/write, Metadata: read, Administration: write for repo creation) over a classic OAuth `repo` scope. See R1.

### 7.2 Vault discovery & creation

- **List vaults:** search the user's repos with topic `gitvault-vault` and a valid `.gitvault/vault.json`.
- **Create vault:** create private repo → initialize `.gitvault/vault.json` (keys wrapped) → add topic → first commit → push.
- **Switching vaults** is a vault-level action inside the app (matches the Figma "vault switcher"), not part of login.

### 7.3 Sync algorithm

1. **Local clone** per vault in the app data dir (`<data>/vaults/<vault_id>/`).
2. Every secret write → write encrypted file → `git commit` (id-only message) → mark dirty.
3. `sync_now` = `fetch` → fast-forward or **per-file merge** → `push`. Auto-sync (setting) triggers after writes and on a timer, with backoff when offline.
4. **Per-file merge rules** (works because of one file per secret):
   - Changed on one side only → take it.
   - Changed on both sides → **conflict**: keep both as versions, surface in UI/CLI (`gitvault sync conflicts`), user picks (`resolve --keep local|remote`).
   - Deleted vs edited → conflict (never silently delete).
5. History for a secret = `git log -- secrets/<id>.enc`, decorated with the encrypted change summary.
6. Offline-first: every operation works offline; sync state is explicit (`synced | pending(n) | offline | conflict(n)`).

### 7.4 Commit signing

The Figma "Verified" label is shown **only** when the commit actually has a valid signature. v1 supports the user's existing git signing config (SSH/GPG) if present; otherwise commits are unsigned and the UI does not claim "Verified".

---

## 8. CLI design — terminal & AI-agent first

Binary: **`gitvault`**. Anything the desktop app can do, the CLI can do, through the **same `Services`** (§2).

### 8.1 Output contract (what makes it AI-friendly)

- **`--json`** on every command → stable, versioned JSON on stdout; human text goes to stderr.
- Default human output is concise tables; `--quiet` prints only the primary value.
- **Never prompts** when stdin is not a TTY or `--non-interactive` is set; missing input is an error with exit code, not a hang.
- **Stable exit codes** (mapped from `ErrorCode`):

| Code | Meaning                                       |
| ---: | --------------------------------------------- |
|    0 | Success                                       |
|    1 | Unexpected / internal error                   |
|    2 | Usage error (bad flags/arguments)             |
|    3 | Not authenticated (run `gitvault auth login`) |
|    4 | Vault locked (run `gitvault unlock`)          |
|    5 | Not found                                     |
|    6 | Conflict                                      |
|    7 | Network / GitHub unavailable                  |
|    8 | Permission denied / policy blocked            |
|    9 | Validation failed                             |

- **Machine-readable errors:** `{"error":{"code":"E_LOCKED","message":"…","hint":"gitvault unlock"}}`.
- **Self-description for agents:**
  - `gitvault commands --json` → full command catalog (names, args, flags, examples).
  - `gitvault schema <type>` → JSON Schema for any output type (generated from Rust DTOs via `schemars`).
  - `gitvault --help` and every subcommand ship **examples**.
  - `docs/cli.md` is generated from the same clap definitions (never hand-edited).
  - `AGENTS.md` / `llms.txt` at repo root describe how an AI tool should use the CLI safely.

### 8.2 Safe-by-default secret handling for AI tools

- `gitvault get NAME` prints **metadata with the value masked**. Raw value requires `--reveal` (or `--raw` for scripting) and is recorded in the audit log.
- **`gitvault run --env production -- <command…>`** injects secrets as environment variables into a child process **without printing them**. This is the recommended path for agents.
- `gitvault inject --in template.tpl --out .env` renders templates (`{{ gitvault://stripe/STRIPE_KEY }}`).
- `--redact` pipe-filter masks known secret values in command output.
- Secrets are never accepted as command-line arguments (they show up in process lists); use `--value-stdin` or `--from-file`.

### 8.3 Command catalog

```
gitvault auth    login | logout | status
gitvault vault   list | create | use | info | delete | lock
gitvault unlock  [--ttl 15m] [--password-stdin] [--export]     # prints GITVAULT_SESSION
gitvault lock
gitvault secret  add | get | list | update | rotate | delete | restore | history | copy | search
gitvault env     list | create | rename | delete
gitvault tag     list | add | remove
gitvault import  env <file>      gitvault export env [--group …] [--environment …]
gitvault generate password|passphrase|pin [--length 24 --symbols …]
gitvault run     [--environment …] [--group …] -- <command…>
gitvault inject  --in <tpl> --out <file>
gitvault sync    now | status | conflicts | resolve
gitvault audit   list [--since 7d --event updated]
gitvault settings get | set
gitvault commands | schema | completions <shell> | version
```

### 8.4 Feature parity matrix (UI ⇄ service ⇄ CLI)

| Capability                   | Core service (use case)                    | Tauri command                | CLI command                            |
| ---------------------------- | ------------------------------------------ | ---------------------------- | -------------------------------------- |
| Sign in with GitHub          | `LoginWithGithub`                          | `auth_login_start/poll`      | `auth login`                           |
| Sign out / who am I          | `Logout`, `CurrentAccount`                 | `auth_logout`, `auth_status` | `auth logout/status`                   |
| List vaults                  | `ListVaults`                               | `vault_list`                 | `vault list`                           |
| Create vault (wizard)        | `CreateVault`                              | `vault_create`               | `vault create`                         |
| Switch vault                 | `OpenVault`                                | `vault_open`                 | `vault use`                            |
| Unlock / lock                | `UnlockVault`, `LockVault`                 | `vault_unlock/lock`          | `unlock`, `lock`                       |
| Add secret (wizard)          | `AddSecret`                                | `secret_add`                 | `secret add`                           |
| List / search                | `ListSecrets`, `SearchSecrets`             | `secret_list/search`         | `secret list/search`                   |
| View / reveal / copy         | `GetSecret`, `RevealSecret`, `CopySecret`  | `secret_get/reveal/copy`     | `secret get [--reveal]`, `secret copy` |
| Edit                         | `UpdateSecret`                             | `secret_update`              | `secret update`                        |
| Rotate                       | `RotateSecret`                             | `secret_rotate`              | `secret rotate`                        |
| Delete / restore             | `DeleteSecret`, `RestoreVersion`           | `secret_delete/restore`      | `secret delete/restore`                |
| Version history              | `SecretHistory`                            | `secret_history`             | `secret history`                       |
| Groups / environments / tags | `Manage*`                                  | `group_*`, `env_*`, `tag_*`  | `group/env/tag …`                      |
| Import / export `.env`       | `ImportEnv`, `ExportEnv`                   | `import_env`, `export_env`   | `import env`, `export env`             |
| Password generator           | `GeneratePassword`                         | `generate_password`          | `generate …`                           |
| Run with secrets             | `RunWithSecrets`                           | — (CLI-native)               | `run`, `inject`                        |
| Sync                         | `SyncNow`, `SyncStatus`, `ResolveConflict` | `sync_*`                     | `sync …`                               |
| Audit log                    | `ListAuditEvents`                          | `audit_list`                 | `audit list`                           |
| Settings                     | `Get/UpdateSettings`                       | `settings_get/set`           | `settings get/set`                     |

`gitvault trust` (guided: asks vault and environments) and `trust add|list|remove` (folder trust) are **cli_only**: it is judged from the current directory of the process that asks to unlock, which the desktop app does not have. A device restricted to folders makes the desktop app ask for the password.

**Parity enforcement:** a test iterates the `Services` use-case registry and asserts each has (a) a Tauri command and (b) a CLI command, or is explicitly listed as `ui_only` / `cli_only` with a reason. CI fails on drift.

### 8.5 Later: MCP server

`gitvault mcp` exposes the same use cases as MCP tools (read-only metadata + `run`-style execution by default; reveal requires explicit policy). Same services, third front door.

---

## 9. Desktop app

### 9.1 Tauri layer (`apps/desktop/src-tauri`)

```
src-tauri/src/
├─ main.rs            # start app only
├─ lib.rs             # builder: plugins, state, command registration
├─ state.rs           # AppState { services: Arc<Services> }
├─ commands/          # one file per area: auth.rs vault.rs secret.rs sync.rs …
├─ events.rs          # typed events: sync-status-changed, vault-locked, device-code
├─ error.rs           # CoreError → ApiError { code, message, hint }
├─ tray.rs, window.rs # OS integration only
└─ capabilities/      # minimal permission files
```

Command handler shape (every handler looks like this):

```rust
#[tauri::command]
#[specta::specta]
pub async fn secret_add(state: State<'_, AppState>, input: AddSecretInput)
    -> Result<SecretSummary, ApiError>
{
    state.services.secrets.add.execute(input).await.map_err(ApiError::from)
}
```

### 9.2 Frontend stack & scaffolding

| Concern        | Choice                                                                                      |
| -------------- | ------------------------------------------------------------------------------------------- |
| Build          | **Vite** + React 18 + TypeScript (strict)                                                   |
| Routing        | **TanStack Router**, **file-based** (`@tanstack/router-plugin` for Vite)                    |
| Data           | **TanStack Query** (all Tauri command results)                                              |
| UI primitives  | **shadcn/ui** (Radix + Tailwind), installed with the **shadcn CLI**, then customized (§9.5) |
| Styling        | **Tailwind CSS v4** (`@tailwindcss/vite`) + CSS variables (theme tokens, §9.6)              |
| Forms          | react-hook-form + zod (`@hookform/resolvers`)                                               |
| UI-only state  | Zustand (small stores)                                                                      |
| Icons / toasts | lucide-react / sonner (shadcn `sonner`)                                                     |

**Bootstrap sequence** (run inside `/home/adhil/projects/gitvault`; nothing is run until the plan is approved):

```bash
# 1. Scaffold the Tauri + React + TS (Vite) app into apps/desktop
pnpm create tauri-app apps/desktop --template react-ts --manager pnpm

# 2. Frontend dependencies
cd apps/desktop
pnpm add @tanstack/react-router @tanstack/react-query zustand react-hook-form zod @hookform/resolvers lucide-react
pnpm add -D @tanstack/router-plugin @tanstack/react-router-devtools @tanstack/react-query-devtools \
           tailwindcss @tailwindcss/vite @types/node vitest @testing-library/react @testing-library/user-event jsdom

# 3. Initialize shadcn (answers: TypeScript yes, Tailwind v4, base color neutral, CSS variables yes)
pnpm dlx shadcn@latest init

# 4. Add the primitives we need (re-runnable; each command copies source into src/components/ui)
pnpm dlx shadcn@latest add button input label textarea select checkbox radio-group switch badge card \
  dialog sheet dropdown-menu popover command tabs table tooltip separator skeleton scroll-area sonner form
```

**Required config**

- `vite.config.ts`: `tanstackRouter({ target: 'react', autoCodeSplitting: true })` **before** `react()`; `tailwindcss()`; alias `@` → `src`.
- `components.json` (written by shadcn): `aliases.components = "@/components"`, `aliases.ui = "@/components/ui"`, `aliases.utils = "@/lib/utils"`, `aliases.hooks = "@/hooks"`, `tailwind.cssVariables = true`.
- `tsconfig.json`: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, path alias `@/*`.
- **Router history:** use **hash history** (`createHashHistory`) so reloads never 404 under Tauri's custom protocol (record as ADR).
- `routeTree.gen.ts` is generated by the plugin: committed, never hand-edited, excluded from lint.

### 9.3 `src/` folder structure

```
apps/desktop/src/
├─ main.tsx                       # mounts <RouterProvider/>, nothing else
├─ routeTree.gen.ts               # GENERATED by TanStack Router
├─ routes/                        # ROUTING ONLY (file-based) — thin files, see §9.4
│  ├─ __root.tsx                  # providers, global error/not-found, devtools, Tauri event bridge
│  ├─ index.tsx                   # "/" → redirect to /sign-in or last vault
│  ├─ _public.tsx                 # layout route: split auth layout (brand panel + outlet)
│  ├─ _public/
│  │  ├─ sign-in.tsx                                  # /sign-in
│  │  └─ sign-in.device.tsx                           # /sign-in/device
│  ├─ _authed.tsx                 # guard: beforeLoad requires a GitHub session
│  └─ _authed/
│     ├─ _wizard.tsx              # layout route: centered wizard layout (step bar + outlet)
│     ├─ _wizard/
│     │  ├─ vaults.new.$step.tsx                      # /vaults/new/$step   (name|repository|password|recovery|done)
│     │  ├─ vaults.$vaultId.unlock.tsx                # /vaults/$vaultId/unlock
│     │  └─ vaults.$vaultId.secrets.new.$step.tsx     # /vaults/$vaultId/secrets/new/$step (details|value|expiry|review|saved)
│     ├─ _app.tsx                 # layout route: app shell (header + sidebar + outlet); guard: vault unlocked
│     └─ _app/
│        ├─ vaults.$vaultId.index.tsx                 # all secrets (search params: environment, group, q, sort)
│        ├─ vaults.$vaultId.secrets.$secretId.tsx     # secret detail panel
│        ├─ vaults.$vaultId.secrets.$secretId.edit.tsx     # edit sheet
│        ├─ vaults.$vaultId.secrets.$secretId.history.tsx  # version history
│        ├─ vaults.$vaultId.secrets.$secretId.delete.tsx   # delete confirmation dialog
│        ├─ vaults.$vaultId.import.tsx                # import .env dialog
│        ├─ tools.generator.tsx                       # password generator
│        ├─ audit.tsx                                 # audit log
│        └─ settings.$tab.tsx                         # settings (general|security|github-sync|appearance|shortcuts)
├─ components/
│  ├─ ui/                         # shadcn primitives (CLI-generated, then customized) — button, input, badge, dialog, sheet…
│  ├─ general/                    # reusable composed components shared by several pages (selectors, fields, layouts)
│  └─ pages/                      # one folder per page; the ONLY components that routes import
├─ api/                           # typed wrappers over Tauri commands — the ONLY place `invoke`/bindings are used
├─ hooks/                         # TanStack Query hooks per area (use-secrets.ts, use-vaults.ts, …)
├─ stores/                        # Zustand stores (ui store, wizard drafts) — in-memory only
├─ schemas/                       # zod schemas (forms, search params, wizard steps)
├─ lib/                           # pure utilities: cn.ts, format-date.ts, mask-value.ts, query-client.ts, router.ts
├─ config/                        # constants, query keys, route/step names
├─ types/                         # GENERATED Tauri/specta bindings (`bindings.ts`) + small hand-written type files
└─ styles/                        # globals.css (Tailwind + theme variables), tokens.css (generated from Figma)
```

### 9.4 Routes folder rules (`src/routes`)

- Route files **only wire things together**: `createFileRoute`, `validateSearch` (zod), `beforeLoad` (guards/redirects), `loader` (prefetch via `queryClient.ensureQueryData`), `pendingComponent` / `errorComponent`, and `component: SomePage`.
- A route file **must not** contain JSX layout/markup, data formatting or business rules, and stays **≤ 40 lines**.
- Each route imports **one** page component from `@/components/pages/...`. Layout routes (`_public`, `_wizard`, `_app`) import their layout from `components/general/layouts`.
- Router context is typed: `{ queryClient, auth }`. Guards:
  - `_authed.beforeLoad` → no GitHub session ⇒ `redirect({ to: '/sign-in' })`.
  - Vault routes open **locked** (see `docs/adr/0001-locked-by-default-vault-browsing.md`): names come from a device-local index and key-needing actions open the unlock dialog (`useRequireUnlock`). There is no unlock route.
  - Backend `vault-locked` (auto-lock) event ⇒ the vault page falls back to the locked view.
- Wizard step is a **path param** validated against a step list; invalid or skipped steps redirect to the first incomplete step.
- Filters (environment, group, query, sort) are **validated search params**, so they survive reloads and are linkable.
- Use typed `<Link>` / `useNavigate`; never build URL strings by hand.

Example of a complete route file:

```tsx
// routes/_authed/_app/vaults.$vaultId.index.tsx
import { createFileRoute } from '@tanstack/react-router'
import { VaultPage } from '@/components/pages/vault-page'
import { vaultFiltersSchema } from '@/schemas/vault-filters'
import { secretsQueryOptions } from '@/hooks/use-secrets'

export const Route = createFileRoute('/_authed/_app/vaults/$vaultId/')({
  validateSearch: vaultFiltersSchema,
  loaderDeps: ({ search }) => search,
  loader: ({ context, params, deps }) =>
    context.queryClient.ensureQueryData(secretsQueryOptions(params.vaultId, deps)),
  component: VaultPage,
})
```

### 9.5 Components folder rules (`src/components`)

**`ui/` — design-system primitives (shadcn)**

- Created with `pnpm dlx shadcn@latest add <component>`; the source lives in our repo, so **we own and modify it**.
- Contains only generic building blocks (Button, Input, Badge, Dialog, Sheet, Select, Switch, Tabs, Table, Tooltip, Sonner…). **No app/domain knowledge**, no data fetching, no imports from outside `lib/`.
- Customize through **`cva` variants and CSS variables**, not ad-hoc class overrides at call sites. GitVault variants: Button `primary` (green), `secondary` (muted + border), `danger`, `ghost`/`link`; sizes `sm` 32px / `md` 40px; radius 6px; Input/Select 40px high; Badge `success | warning | danger | info | done | neutral`.
- Record every deviation from upstream in `components/ui/CHANGES.md` so upgrades (`shadcn@latest add <x> --overwrite`) can be re-applied deliberately.

**`general/` — reusable composed components**

- Built from `ui/` primitives, **used by two or more pages**, still free of page-specific logic. Examples:
  - Selectors: `vault-selector`, `environment-select`, `group-select`, `secret-type-picker`, `tag-input`, `date-picker`.
  - Fields: `text-field`, `select-field`, `password-field` (mask / reveal / copy), `copy-button`.
  - Patterns: `wizard-steps`, `confirm-dialog`, `empty-state`, `status-badge`, `page-header`, `data-table`, `key-value-list`, `callout`.
  - Layouts (`general/layouts/`): `auth-split-layout`, `wizard-layout`, `app-shell` (header + sidebar).
- May use hooks for data (`vault-selector` uses `useVaults`), but receives behavior through props wherever possible.

**`pages/` — page-level components**

- One folder per page; the folder's `index.ts` exports **only the page component**. Page-specific parts stay inside the folder; **promote to `general/` when a second page needs it**.
- Pages compose `general/` + `ui/`, call hooks, own the page's form wiring and wizard step content. They **never** import other pages.

```
components/pages/
├─ sign-in-page/            sign-in-page.tsx  permission-list.tsx  index.ts
├─ authorize-device-page/   authorize-device-page.tsx  device-code-box.tsx  index.ts
├─ vault-page/              vault-page.tsx  vault-toolbar.tsx  secret-list.tsx  secret-row.tsx
│                           secret-detail-panel.tsx  use-vault-filters.ts  index.ts
├─ create-vault-page/       create-vault-page.tsx  steps/ (name-step.tsx repository-step.tsx password-step.tsx recovery-step.tsx done-step.tsx)
├─ unlock-page/             unlock-page.tsx  index.ts
├─ add-secret-page/         add-secret-page.tsx  steps/ (details-step.tsx value-step.tsx expiry-step.tsx review-step.tsx saved-step.tsx)
├─ edit-secret-sheet/  secret-history-page/  import-env-dialog/  delete-secret-dialog/
├─ generator-page/  audit-log-page/  settings-page/ (tabs/ security-tab.tsx github-sync-tab.tsx appearance-tab.tsx …)
```

**Import boundaries (enforced by ESLint `boundaries` + dependency-cruiser):**

```
routes ─► components/pages ─► components/general ─► components/ui ─► lib
              │                      │
              └──────────────► hooks ─► api ─► types (bindings)
```

- `ui` imports only `lib`. `general` imports `ui`, `hooks`, `lib`, `schemas`. `pages` import `general`, `ui`, `hooks`, `stores`, `schemas`, `lib`.
- `api` is imported **only by `hooks`** (and route loaders through hooks' query options). Components never call `invoke` or `api/*` directly.
- No circular imports; no imports from `routes/` except typed router APIs and route types.

### 9.6 Theming (Figma tokens → shadcn variables)

- Source of truth: Figma variable collection **"GitHub Theme"** (modes **Dark** / **Light**). `packages/tokens` generates `tokens.css`; `styles/globals.css` maps them onto shadcn's CSS variables.
- Dark mode uses the shadcn convention: `.dark` class on `<html>`, driven by `next-themes` (`ThemeProvider attribute="class"`, which works in plain React/Vite); setting is `dark | light | system`.

| shadcn variable                                                                            | Figma token                                                             | Dark                 | Light                |
| ------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------- | -------------------- | -------------------- |
| `--background`                                                                             | bg/canvas                                                               | `#010409`            | `#ffffff`            |
| `--foreground`                                                                             | fg/default                                                              | `#e6edf3`            | `#1f2328`            |
| `--card`                                                                                   | bg/card                                                                 | `#151b1b`            | `#f6f8fa`            |
| `--muted`                                                                                  | bg/muted                                                                | `#212830`            | `#eaeef2`            |
| `--muted-foreground`                                                                       | fg/muted                                                                | `#9198a1`            | `#59636e`            |
| `--accent`                                                                                 | bg/hover                                                                | `#2a313c`            | `#d0d7de`            |
| `--border` / `--input`                                                                     | border/default                                                          | `#3d444d`            | `#d1d9e0`            |
| `--ring`                                                                                   | accent/fg                                                               | `#2f81f7`            | `#0969da`            |
| `--primary` / `--primary-foreground`                                                       | success/emphasis / fg/onEmphasis                                        | `#238636` / `#fff`   | `#1f883d` / `#fff`   |
| `--secondary`                                                                              | bg/muted                                                                | `#212830`            | `#eaeef2`            |
| `--destructive`                                                                            | danger/emphasis                                                         | `#da3633`            | `#cf222e`            |
| extra: `--inset`, `--subtle`, `--success`, `--warning`, `--info`, `--done`, `--brand-line` | inset, subtle, success/fg, attention/fg, accent/fg, done/fg, brand/line | see Foundations page | see Foundations page |

- Components never hard-code colors; they use semantic Tailwind utilities (`bg-card`, `text-muted-foreground`, `border-border`).
- The **green header line**, 6px radius, mono font for secret names (JetBrains Mono) and Inter for UI are set once in `globals.css`.

### 9.7 UI data flow & state

- **Server state = TanStack Query.** Query keys live in `config/query-keys.ts`; every `api/*` function has one `hooks/*` wrapper (`useSecrets`, `useAddSecret`, …). Mutations invalidate by key.
- **Loaders prefetch** with `ensureQueryData` so pages render with data; `pendingComponent` shows `Skeleton`s; `errorComponent` shows a typed `ApiError` (`code`, `message`, `hint`).
- **UI state:** `useState` first; Zustand only for cross-route UI state (theme, sidebar, wizard drafts).
- **Wizards** (create vault, add secret): one route per step, shared draft store (`stores/wizard-draft.ts`), per-step zod schema, Back/Continue navigate between routes. Drafts are **in-memory only**, cleared on finish, cancel, vault lock or app quit. **Secret values and passwords are never persisted** (no localStorage/sessionStorage, not in Query cache, not in URLs).
- **Revealing a secret:** `useRevealSecret` calls `secret_reveal`, keeps the value in component state, and clears it on hide, timeout (setting), blur or lock.
- **Backend events** (`sync-status-changed`, `vault-locked`, `device-code`) are subscribed once in `routes/__root.tsx` via a `useTauriEvents` hook that updates Query cache / navigates.
- **Errors:** map `ApiError.code` → user-facing message in one place (`lib/api-error.ts`); toasts via `sonner`.

### 9.8 UI coding conventions

- File names **kebab-case**; component name PascalCase matching the file (`secret-row.tsx` → `SecretRow`); page components end in `Page`; props typed as `<Component>Props`.
- **Named exports only** (route files export `Route`); barrel `index.ts` allowed **only** in `components/pages/<page>/`.
- One component per file; component ≤ **150 lines**, route file ≤ **40 lines**, hook ≤ **60 lines**.
- Business rules stay in Rust. React validates input shape (zod) for UX only; the core re-validates.
- Every async UI shows loading, empty and error states.
- Accessibility: keyboard reachable, visible focus, labelled controls, dialogs trap focus (Radix gives this for free), WCAG AA contrast.

### 9.9 UI testing & documentation

- **Storybook** for `ui/` and `general/` (light + dark), mirroring the Figma Components page; visual regression via Playwright.
- **Vitest + Testing Library** for hooks, general components and page behavior with a mocked Tauri `invoke` (`@tauri-apps/api/mocks`).
- **Route tests** use `createMemoryHistory` + a test router context to assert guards/redirects (signed-out → `/sign-in`, locked → unlock).
- **Playwright e2e** on the critical flows with a mocked backend; `tauri-driver` smoke test on the real binary.

### 9.10 Figma → route map

| Figma page / screen                                                            | Route                                        | Page component                        |
| ------------------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------- |
| Auth · Sign in with GitHub                                                     | `/sign-in`                                   | `SignInPage`                          |
| Auth · Authorize device                                                        | `/sign-in/device`                            | `AuthorizeDevicePage`                 |
| Vault · All secrets (+ switcher, empty state)                                  | `/vaults/$vaultId`                           | `VaultPage`                           |
| Vault · Create 1–5 (Name → Repository → Master password → Recovery key → Done) | `/vaults/new/$step`                          | `CreateVaultPage`                     |
| Vault · Unlock                                                                 | `/vaults/$vaultId/unlock`                    | `UnlockPage`                          |
| Add secret 1–5 (Details → Value → Expiry → Review → Saved)                     | `/vaults/$vaultId/secrets/new/$step`         | `AddSecretPage`                       |
| Secret detail / Edit drawer                                                    | `/vaults/$vaultId/secrets/$secretId[/edit]`  | `VaultPage` panel / `EditSecretSheet` |
| Version history                                                                | `/vaults/$vaultId/secrets/$secretId/history` | `SecretHistoryPage`                   |
| Delete confirmation                                                            | `/vaults/$vaultId/secrets/$secretId/delete`  | `DeleteSecretDialog`                  |
| Import .env                                                                    | `/vaults/$vaultId/import`                    | `ImportEnvDialog`                     |
| Tools · Password generator                                                     | `/tools/generator`                           | `GeneratorPage`                       |
| Tools · Audit log                                                              | `/audit`                                     | `AuditLogPage`                        |
| Settings · Security / GitHub & Sync / Appearance                               | `/settings/$tab`                             | `SettingsPage`                        |

---

## 10. Coding rules

These rules apply to humans **and** AI coding tools. They are mirrored in `AGENTS.md` and enforced in CI wherever possible.

### 10.1 Universal principles

1. **SOLID**
   - _Single responsibility:_ one reason to change per function, file and module.
   - _Open/closed:_ extend through new use cases/adapters, not by editing stable ones.
   - _Liskov:_ every adapter must honor its port's contract (shared contract tests).
   - _Interface segregation:_ small, focused ports (`KeyStore`, `Clock`), not a mega-trait.
   - _Dependency inversion:_ core depends on traits; infra implements them.
2. **DRY, but not at any cost** — duplicate twice, abstract on the third time. Never share code that only looks similar.
3. **KISS & YAGNI** — build what a current screen/command needs. No speculative generality.
4. **Separation of concerns** — domain rules, orchestration, I/O and presentation never mix.
5. **Composition over inheritance. Immutability by default.**
6. **Pure functions where possible;** side effects at the edges.
7. **Make illegal states unrepresentable** — newtypes, enums, parse-don't-validate.
8. **Fail fast, fail typed** — return `Result`, never swallow errors, no sentinel values.
9. **Law of Demeter / tell, don't ask** — don't reach through objects.
10. **Boy-scout rule** — leave code a little cleaner than you found it (within PR scope).

### 10.2 Size and shape limits (CI-enforced where possible)

| Item                  | Limit                                                              |
| --------------------- | ------------------------------------------------------------------ |
| Function / method     | **≤ 30 lines**, one level of abstraction, ideally ≤ 15             |
| Function parameters   | **≤ 3** (more ⇒ a typed input struct/object)                       |
| Nesting depth         | **≤ 3** (use early returns / guard clauses)                        |
| Cyclomatic complexity | **≤ 8** per function                                               |
| File                  | **≤ 300 lines** (hard stop 400); one primary concept per file      |
| React component       | **≤ 150 lines** JSX+logic; extract subcomponents/hooks beyond that |
| Module/folder         | If a folder has > ~10 files, group into sub-folders by concept     |
| Boolean parameters    | Avoid; use enums / separate functions                              |

### 10.3 Naming

- Say what it **is** or **does**; no abbreviations beyond universal ones (`id`, `url`, `repo`).
- Functions are verbs (`unlock_vault`, `parseSecretName`); types are nouns; booleans read as questions (`is_locked`, `hasExpired`).
- Rust: `snake_case` fns/modules, `PascalCase` types, `SCREAMING_SNAKE_CASE` consts. TS: `camelCase` fns/vars, `PascalCase` components/types, **`kebab-case` file names** (`secret-detail-panel.tsx`).
- One exported component/class per file, file named after it.
- No `utils.ts` / `helpers.ts` / `common.ts` dumping grounds — name files by purpose (`mask-value.ts`).
- No magic numbers or strings: named constants in `config/` or `constants.rs`.

### 10.4 Functions and files — how to split

- If you need "and" to describe a function, split it.
- If a comment explains _what_ a block does, extract it into a function named after the comment.
- Orchestration functions read like a table of contents; details live in small helpers.
- Rust: organize by use case/domain concept, not by technical type. Frontend: follow the fixed structure in §9.3 (`routes/`, `components/{ui,general,pages}`, `api/`, `hooks/`).
- Public surface is explicit: Rust `pub(crate)` by default; in the frontend only `components/pages/<page>/index.ts` barrels are allowed and they export just the page.

### 10.5 Errors, logging, comments

- Rust libraries: `thiserror` enums with a stable `ErrorCode`; binaries: `anyhow` at the very top only. **No `unwrap()`/`expect()`/`panic!` in non-test code** (clippy-denied); `?` everywhere.
- TS: no `any`; no non-null `!` assertions without a justification comment; handle `Result`/error states in every query UI (loading / empty / error).
- Never log secret values, tokens, passwords, recovery keys, or full request bodies.
- Comments explain **why**, not **what**. Public Rust items have `///` docs with an example for non-obvious ones. Remove dead and commented-out code.
- TODOs must reference an issue: `// TODO(#123): …`.

### 10.6 Rust-specific

- `#![forbid(unsafe_code)]` in `crypto` and `core`.
- `cargo clippy --all-targets -- -D warnings` plus selected pedantic lints (`unwrap_used`, `expect_used`, `panic`, `dbg_macro`, `todo`, `cognitive_complexity`, `too_many_lines`, `too_many_arguments`).
- Secrets use `secrecy`/`zeroize` types; custom `Debug` prints `[REDACTED]`.
- Async only at I/O boundaries; domain code is sync and pure.
- Public APIs take `&str`/slices, return owned DTOs; avoid leaking `Arc<Mutex<…>>` in signatures.
- Every `pub` use case has at least: success test, validation failure test, and port-failure test.

Example — the shape of every use case:

```rust
/// Adds a new secret to the unlocked vault.
pub struct AddSecret {
    vaults: Arc<dyn VaultRepository>,
    sessions: Arc<SessionManager>,
    ids: Arc<dyn IdGenerator>,
    clock: Arc<dyn Clock>,
    audit: Arc<dyn AuditLog>,
}

impl AddSecret {
    pub async fn execute(&self, input: AddSecretInput) -> Result<SecretSummary, CoreError> {
        let key = self.sessions.vault_key(&input.vault_id)?;        // guard: must be unlocked
        let secret = self.build_secret(input)?;                      // domain validation
        self.vaults.save_secret(&input.vault_id, &secret, &key).await?;
        self.audit.record(AuditEvent::created(&secret, self.clock.now())).await?;
        Ok(SecretSummary::from(&secret))
    }

    fn build_secret(&self, input: AddSecretInput) -> Result<Secret, CoreError> { /* … */ }
}
```

### 10.7 TypeScript / React-specific

- `strict: true`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`; ESLint `no-explicit-any`, `consistent-type-imports`, `react-hooks/*`, `jsx-a11y`.
- Function components + hooks only. Props typed with named interfaces; no prop drilling past 2 levels (use composition or a feature hook).
- **No business logic in components.** Components render; hooks orchestrate; Rust decides. Routes only wire (§9.4).
- Hooks are small and single-purpose (`useSecretList`, `useRevealSecret`, `useWizardStep`).
- Forms: schema in `model/` (zod), reused for validation and types.
- No inline styles; Tailwind classes via tokens; extract repeated class sets into `components/ui` or `components/general` components (not `@apply` soup).
- Side effects only in effects/handlers; keep render pure. Memoize only with evidence.
- Test ids via accessible roles first; `data-testid` only as last resort.

Example — API wrapper + hook (`api/` is the only place Tauri commands are called):

```ts
// api/secrets.ts  (the only layer that touches Tauri bindings)
import { commands } from '@/types/bindings'
export const addSecret = (input: AddSecretInput) => commands.secretAdd(input)

// hooks/use-secrets.ts
export function useAddSecret() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: addSecret,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: secretKeys.all }),
  })
}
```

### 10.8 Testing rules

- **Test pyramid:** many fast unit tests, some integration tests, few e2e.
- Tests are named by behavior: `unlock_fails_with_wrong_password`.
- Arrange-Act-Assert, one behavior per test, no logic in tests.
- No real network/keychain/time in unit tests — use fakes from `gitvault-infra::testing`.
- Crypto: known-answer test vectors + round-trip + tamper-detection tests.
- Fix a bug ⇒ add the regression test first.
- Targets: `crypto` ≥ 95%, `core` ≥ 85%, `infra` ≥ 70%, frontend features ≥ 70% (logic-heavy hooks/models higher).

### 10.9 Git, PRs and review

- **Conventional Commits** (`feat(secrets): add expiry reminders`), enforced by commitlint.
- Trunk-based: short-lived branches from `main` (`feat/…`, `fix/…`), squash merge, no direct pushes to `main`.
- PRs ≤ ~400 changed lines, one concern each, include screenshots for UI changes and link the Figma frame.
- Required checks: format, lint, type-check, tests, boundaries, parity test, `cargo deny`, `pnpm audit`.
- Review checklist: single responsibility? tests? secrets handling? error paths? naming? docs/ADR updated?
- Architecture-affecting changes need an **ADR** in `docs/adr/` (template in §15).

### 10.10 Definition of Done (per feature)

- [ ] Core use case implemented and unit-tested (success, validation, port-failure).
- [ ] CLI command implemented with `--json`, exit codes, examples, snapshot test.
- [ ] Tauri command + typed binding generated; UI built from design-system components.
- [ ] Parity test passes.
- [ ] No secrets in logs/cache/URLs; accessibility checked.
- [ ] Docs (`docs/cli.md` regenerated, ADR if needed) and Figma frame linked.
- [ ] CI green on macOS, Windows, Linux.

### 10.11 Enforcement tooling

| Rule                | Tool                                                                                              |
| ------------------- | ------------------------------------------------------------------------------------------------- |
| Formatting          | `rustfmt`, Prettier/Biome (pre-commit via lefthook)                                               |
| Lints & size limits | `clippy`, ESLint (`max-lines`, `max-lines-per-function`, `max-params`, `complexity`, `max-depth`) |
| Layering            | Cargo crate graph; ESLint boundaries / dependency-cruiser; `cargo-deny` bans                      |
| No unwrap / any     | clippy deny, `@typescript-eslint/no-explicit-any`                                                 |
| Commit format       | commitlint                                                                                        |
| Parity              | custom test over `Services` registry                                                              |
| Dependency safety   | `cargo-deny`, `cargo-audit`, `pnpm audit`, Renovate                                               |
| Secrets in repo     | gitleaks pre-commit + CI                                                                          |

---

## 11. Testing strategy

| Level              | Scope                                                                    | Tooling                                          |
| ------------------ | ------------------------------------------------------------------------ | ------------------------------------------------ |
| Unit               | domain types, crypto, merge rules, mask/format helpers                   | `cargo nextest`, Vitest                          |
| Contract           | each adapter satisfies its port (fs, git, keychain, GitHub fake)         | shared test suites per port                      |
| Integration (core) | use cases wired to in-memory fakes and **temp git repos**                | `gitvault-infra::testing`                        |
| CLI                | commands, `--json` snapshots, exit codes, non-TTY behavior               | `assert_cmd`, `insta`                            |
| Contract (UI↔core) | generated TS bindings compile; JSON Schemas stable                       | CI codegen diff check                            |
| Frontend           | components, hooks, wizard flows                                          | Vitest + Testing Library, mocked `invoke`        |
| E2E                | critical flows: sign-in (mock GitHub) → create vault → add secret → sync | Playwright (mock backend) + `tauri-driver` smoke |
| Security           | tamper tests, wrong-password timing sanity, fuzz of envelope parser      | `cargo-fuzz` (envelope + `.env` parser)          |
| Visual             | components vs Figma tokens, light/dark                                   | Storybook + visual regression (Playwright)       |

---

## 12. CI/CD & release

**CI (`ci.yml`)** on every PR: format → lint → type-check → unit → integration → CLI snapshots → frontend tests → boundaries → parity → codegen-drift check → `cargo deny`/`audit` → build (Linux, macOS, Windows matrix).

**Release (`release.yml`)**: `release-please` versioning → `tauri-action` builds signed installers (`.dmg`, `.msi`, `.AppImage/.deb`) → publish CLI archives + checksums (and Homebrew/Scoop/`cargo install`) → signed updater manifest → SBOM attached.

**Environments:** local dev (`pnpm tauri dev`, `cargo run -p gitvault-cli`), CI, release. A **fake GitHub server** (wiremock) is used for tests and offline development.

---

## 13. Roadmap

Each phase ends in something runnable and demonstrable.

### Phase 0 — Foundations (≈ 1 week)

- Monorepo, Cargo + pnpm workspaces, CI skeleton, lint/format/commit hooks, `AGENTS.md`, ADR 0001 (this architecture).
- Design tokens package generated from Figma variables (Dark/Light).
- Scaffold `apps/desktop` (Tauri + React/Vite), TanStack Router plugin, Tailwind v4, **shadcn init + add** the primitives in §9.2, customized to the GitHub theme (matches Figma Components page).
- **Exit:** `pnpm dev` opens an empty themed window; `gitvault --version` runs; CI green.

### Phase 1 — Core vertical slice, **no GitHub yet** (≈ 2–3 weeks)

- `gitvault-crypto` (Argon2id, AES-GCM, key wrapping, recovery key) with test vectors.
- Domain model + ports + `AddSecret/GetSecret/ListSecrets/Update/Delete`, local filesystem vault repository, `SessionManager`.
- CLI: `vault create`, `unlock`, `secret add/get/list/update/delete`, `--json`, exit codes, `commands`, `schema`.
- **Exit:** create a local vault and manage secrets entirely from the terminal; encrypted files verified by tamper tests.

### Phase 2 — GitHub identity & sync (≈ 2 weeks)

- Device-flow login, keychain token, repo discovery (topic), repo creation, `git2` transport, commit/push/pull, per-file merge, conflict model.
- CLI: `auth *`, `vault list/use`, `sync *`.
- **Exit:** two machines share one vault via a private repo; conflicts are detected and resolvable from CLI.

### Phase 3 — Desktop app (≈ 4 weeks)

Built screen-by-screen from Figma (route → page component → hooks → existing services):

1. Sign in + Authorize device
2. Vault shell (sidebar, list, detail, switcher, empty state)
3. Create-vault wizard (5 pages) + Unlock
4. Add-secret wizard (5 pages), Edit drawer, History, Import `.env`, Delete confirm
5. Password generator, Audit log, Settings
6. Light/dark theming, accessibility pass

- **Exit:** all Figma screens implemented; parity test green; e2e happy path passes.

### Phase 4 — Hardening (≈ 2 weeks)

- Auto-lock, clipboard clearing, reveal timeouts, biometric unlock, secret expiry reminders, audit log completeness, `.env` import/export edge cases, offline/backoff behavior, fuzzing, security review of Tauri capabilities.
- **Exit:** threat-model checklist complete; no `unwrap` in core; dependency audits clean.

### Phase 5 — AI-friendly & release (≈ 2 weeks)

- `run`, `inject`, `--redact`, `completions`, generated `docs/cli.md`, `llms.txt`, **MCP server** (`gitvault mcp`).
- Signed installers, auto-updater, Homebrew/Scoop, docs site, v1.0.
- **Exit:** an AI agent can discover commands via `gitvault commands --json`, run a task with secrets injected, and never see raw values.

---

## 14. Risks & open decisions

| #      | Item                                                                                                                                                                                                        | Impact                                   | Proposed resolution                                                                                                                                                                                            |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **R1** | **GitHub permissions vs Figma copy.** Creating/pushing a private vault repo needs repository write access, but the sign-in screen says only `read:user`, `user:email` and "no access to your repositories". | Misleading consent UI; trust issue.      | Use a **GitHub App** (fine-grained: Contents RW, Administration W for creation, Metadata R) installable on selected repos; update the sign-in screen text to list the real permissions. Decide before Phase 2. |
| R2     | Master password at creation vs "GitHub login only".                                                                                                                                                         | UX confusion.                            | Keep: GitHub = identity/sync, master password = encryption (set once per vault in the wizard). Already reflected in the design. Optional later: biometric/keychain-stored key for passwordless unlock.         |
| R3     | "Verified" commit badge.                                                                                                                                                                                    | False claim if unsigned.                 | Show only when signature validates (see §7.4).                                                                                                                                                                 |
| R4     | Linux keychain availability (Secret Service).                                                                                                                                                               | Login/session storage may fail headless. | Fallback to encrypted file store with explicit user opt-in; document clearly.                                                                                                                                  |
| R5     | Per-file merge conflicts on rotating secrets across devices.                                                                                                                                                | Data loss risk if mishandled.            | Never auto-resolve both-sides-edited; keep both versions; cover with property tests.                                                                                                                           |
| R6     | Search over encrypted names.                                                                                                                                                                                | Performance with large vaults.           | Build an in-memory index at unlock; optional encrypted index file.                                                                                                                                             |
| R7     | Rate limits / GitHub outages.                                                                                                                                                                               | Sync failures.                           | Offline-first design, exponential backoff, explicit sync state in UI/CLI.                                                                                                                                      |
| R8     | Design assets still say "Vaultkeeper".                                                                                                                                                                      | Inconsistent branding.                   | Rename in Figma and tokens during Phase 0.                                                                                                                                                                     |
| R9     | Scope creep (MCP, sharing, mobile).                                                                                                                                                                         | Delays v1.                               | Hold behind roadmap phases; require ADR to pull items forward.                                                                                                                                                 |

---

## 15. Appendix

### A. ADR template (`docs/adr/NNNN-title.md`)

```
# NNNN — Title
Status: proposed | accepted | superseded by NNNN
Context: what problem, constraints, forces
Decision: what we chose
Consequences: positive, negative, follow-ups
Alternatives considered: …
```

### B. `AGENTS.md` (for AI coding tools) — outline

1. Read `docs/architecture.md` §2, §4, §10 first.
2. Never put business logic in `src-tauri`, `gitvault-cli`, or React components.
3. New capability ⇒ core use case → CLI command → Tauri command → UI, in that order.
4. Respect size limits and naming rules; run `just check` before finishing.
5. Never log, print, commit or cache secret values.
6. Update docs/ADR when changing architecture.

### C. Dev commands (via `just` or `pnpm` scripts)

```
just dev          # tauri dev (desktop)
just cli -- …     # run the CLI
just check        # fmt + lint + typecheck + tests + boundaries + parity
just codegen      # regenerate TS bindings, JSON schemas, CLI docs, tokens
just e2e          # Playwright
```

### D. First tickets to create

1. Scaffold workspace + CI + hooks (Phase 0).
2. Generate design tokens from Figma variables.
3. `gitvault-crypto`: Argon2id + AES-GCM + wrapping + vectors.
4. Core domain types + ports + fakes.
5. `AddSecret`/`GetSecret` + CLI `secret add/get` with `--json`.
6. Parity test harness.
7. ADR 0001: architecture; ADR 0002: vault format; ADR 0003: GitHub App vs OAuth.

---

## 16. As built (v0.1) — where the implementation differs from the plan

The plan above is the design; these are the deliberate differences made while building it.

| Area              | Plan                          | As built                                                                                                                          | Why                                                                                                                                |
| ----------------- | ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Core I/O          | async ports                   | **synchronous** ports and services; Tauri runs them with `spawn_blocking`                                                         | Far simpler, no runtime in the core, CLI is naturally blocking.                                                                    |
| Git               | `git2` (libgit2)              | the **`git` command line** (`gitvault-infra/src/git`), token passed through `GIT_CONFIG_*` environment variables, never arguments | No C build dependency; honors the user's git/proxy config. Requires `git` on PATH.                                                 |
| Local state       | SQLite audit log, OS keychain | **JSONL audit log** and a **private (0600) JSON state file** for the GitHub token, settings and sessions                          | Fewer dependencies. The OS keychain is the next hardening step (the `KeyValueStore` port is ready for it).                         |
| Type sharing      | `tauri-specta`                | **`ts-rs`** (`pnpm gen:types`) plus `schemars` JSON Schemas in the CLI                                                            | Simple, stable derive macros on the DTOs.                                                                                          |
| Core layout       | one file per use case         | one **service per area**, one file per operation group (`application/secrets/{crud,query,history,transfer}.rs`)                   | Same separation with far less boilerplate.                                                                                         |
| GitHub permission | GitHub App, narrow scopes     | OAuth **device flow with the `repo` scope** (or a personal access token)                                                          | Simplest way to create a private repo and push. The sign-in screen says so. Moving to a GitHub App is still recommended (risk R1). |
| Dark mode         | own provider                  | `next-themes` with the `.dark` class                                                                                              | Works in plain React/Vite and with shadcn.                                                                                         |
| Folders           | Folder                        | **None** — groups were removed; the vault (one GitHub repo) is the unit of access, Environment and Tags stay as labels            | Product decision after design review.                                                                                              |
| Auth flow         | master password at login      | **GitHub sign-in only**; the master password is per vault, set in the create-vault wizard                                         | Product decision.                                                                                                                  |

### Lint limits actually enforced

- Rust (`clippy.toml`): functions ≤ 40 lines, ≤ 5 parameters plus `self`, cognitive complexity ≤ 12, no `unwrap`/`expect`/`panic` outside tests.
- TypeScript: plain functions ≤ 30 lines (hooks ≤ 60, React components ≤ 150), ≤ 3 parameters, nesting ≤ 3, complexity ≤ 8 (components ≤ 12), files ≤ 300 lines, route files ≤ 40 lines.
- Exemptions are explicit: the browser-only mock backend (`src/api/mock`) and a few `#[allow]` attributes with a comment saying why.

### Not built yet

- OS keychain storage for the GitHub token and sessions.
- Biometric unlock, signed auto-updater, installers/release workflow (`tauri-action`), Homebrew/Scoop packages.
- Commit signing / the "Verified" badge (commits are unsigned; the UI never claims "Verified").
- Playwright and `tauri-driver` end-to-end tests (the app is covered by Vitest flows against the mock backend and by Rust integration tests with real git).
- A live test against github.com (all GitHub calls are unit-tested for parsing; the repo flows are tested against local bare repositories).
