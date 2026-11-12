# Signed apt repository

Every release publishes a GPG-signed apt repository, so Ubuntu/Debian users install and update
GitVault with `apt`, and apt verifies the signature on every update.

## For users

```bash
sudo curl -fsSL https://github.com/adhil72/gitvault-website/releases/latest/download/gitvault-archive-keyring.gpg \
  -o /usr/share/keyrings/gitvault-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/gitvault-archive-keyring.gpg] https://github.com/adhil72/gitvault-website/releases/latest/download ./" \
  | sudo tee /etc/apt/sources.list.d/gitvault.list
sudo apt update
sudo apt install git-vault
```

`signed-by` limits the key to this one repository. The armored key is also published as
`gitvault-archive-keyring.asc`; check that its fingerprint is

```
2F22 86FD 69D5 C056 80FC  12BF BCCE 9E57 C812 62F0
```

with `gpg --show-keys --fingerprint gitvault-archive-keyring.asc` before trusting it.

Only the latest release is offered through apt. Older versions are on the releases page.

## How it works

The release workflow's `publish` job runs `scripts/build-apt-repo.sh`, which builds a _flat_
repository: `Packages`, `Packages.gz`, `Release`, `InRelease` (clear-signed) and `Release.gpg`
(detached). They are uploaded as assets of the public release, next to `GitVault.deb`, so the URL
`releases/latest/download` is the repository root. The script verifies its own signatures with
`gpgv` and fails the release if they do not check out.

## One-time setup (maintainer)

Use a key made only for this repository, with a passphrase.

```bash
gpg --quick-generate-key "GitVault Archive <you@example.com>" ed25519 sign 3y
gpg --list-keys --fingerprint "GitVault Archive"      # the fingerprint published above
gpg --armor --export-secret-keys "GitVault Archive"   # paste into the APT_GPG_PRIVATE_KEY secret
```

Add two secrets to the source repository (Settings → Secrets and variables → Actions):

| Secret                | Value                                 |
| --------------------- | ------------------------------------- |
| `APT_GPG_PRIVATE_KEY` | The armored private key printed above |
| `APT_GPG_PASSPHRASE`  | The key's passphrase                  |

Then keep an offline backup of the key and its revocation certificate (`~/.gnupg/openpgp-revocs.d/`).
Without the secrets the workflow skips the apt step with a warning and releases as before.

## Renewing or rotating the key

The key above expires after 3 years. Before then, extend it (`gpg --quick-set-expire`) and update
the secret, which keeps the same fingerprint. If the key is ever compromised, revoke it, publish a
new one, and tell users to replace the keyring file.
