# GitVault

**Zero `.env` files. Pure CLI. Powered exclusively by `git` and `gh`.**

GitVault eliminates plaintext `.env` files from your repositories and local disks. Every secret stays encrypted in a private GitHub repository (`gitvault-store`) managed through your existing `git` and `gh` CLI authentication—with zero internal account management, zero desktop app, and zero language SDKs.

## Prerequisites

Both `git` and the GitHub CLI (`gh`) must be installed and signed in:

```bash
gh auth login
```

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/adhil72/gitvault/main/scripts/install.sh | sh
```

Or build from a local checkout:

```bash
./scripts/install.sh
# or directly with cargo:
cargo install --git https://github.com/adhil72/gitvault --locked
```

Add the shell wrapper to `~/.bashrc` or `~/.zshrc` (or `~/.config/fish/config.fish`) so `gitvault load` updates your current shell session directly in RAM:

```bash
# bash / zsh
eval "$(gitvault shell-init)"

# fish
gitvault shell-init fish | source
```

## 60-Second Workflow

### 1. Initialize a project (`gitvault init`)
```bash
cd ~/projects/my-api
gitvault init
```
Interactive prompts ask you to pick or create a project and default environment, and write a tiny non-secret `gitvault.json`:
```json
{
  "project": "my-api",
  "environment": "development"
}
```
No `.env` file is created.

### 2. Configure variables & environments (`gitvault config`)
```bash
gitvault config
```
Interactive menu lets you select **Config variables** → select or create an environment → add, view, delete, copy, or migrate legacy `.env` files (`--delete-source` removes the old plaintext `.env`).

Direct CLI shortcuts are also available:
```bash
gitvault config set DATABASE_URL=postgres://localhost/app -p my-api -e development
gitvault config get DATABASE_URL
gitvault config list --reveal
gitvault config rm DATABASE_URL
gitvault config import .env --delete-source
gitvault config env create staging
gitvault config env clone development staging
```

### 3. Load secrets in memory (`gitvault load`)
Never write a `.env` file again:

```bash
# Inject into a child process directly (no .env file):
gitvault load -- npm run dev
gitvault load -p my-api -e production -- ./deploy.sh

# Load into your current shell session (with `eval "$(gitvault shell-init)"`):
gitvault load
gitvault load -p my-api -e production
gitvault unload

# Export or JSON on stdout for scripts:
eval "$(gitvault load -p my-api -e production --export)"
gitvault load -p my-api -e production --json
```

## Security Design

- **Key hierarchy:** Master password → **Argon2id** (64 MiB, 3 iterations) → KEK wraps a random 256-bit `VaultKey` (plus a 160-bit `RecoveryKey` via HKDF-SHA256).
- **Authenticated encryption:** One encrypted blob per environment (`projects/<project>/envs/<env>.enc`) encrypted with **AES-256-GCM** using fresh 96-bit nonces and associated data `gitvault:v2:<project>:<environment>` to prevent cross-environment or cross-project tampering.
- **Hidden local store & key cache:** Vault repository lives at `~/.local/share/gitvault/store` (`0700`); unlocked `VaultKey` cache lives separately at `~/.local/share/gitvault/session.key` (`0600`) and is wiped by `gitvault lock`.
- **No `.env` files on disk:** Secrets are decrypted directly into process RAM (`Command::envs` or shell `eval`).

## License

MIT — see [LICENSE](LICENSE).
