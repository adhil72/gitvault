# ADR 0003: Signed apt repository served from release assets

## Status

Accepted

## Context

Ubuntu users installed a `.deb` by hand, with nothing but `SHA256SUMS` to verify it and no
automatic updates. The source repository is private; releases go to a public repository.

## Decision

- Each release builds a flat apt repository and signs it with a dedicated GPG key
  (`scripts/build-apt-repo.sh`, run in the `publish` job).
- The files are uploaded as assets of the public release, so
  `https://github.com/adhil72/gitvault-website/releases/latest/download` is the repository root.
  The `.deb` is published under the stable name `GitVault.deb` so `Filename:` never changes.
- The private key and passphrase live in the `APT_GPG_PRIVATE_KEY` and `APT_GPG_PASSPHRASE`
  secrets. Without them the step is skipped, like Windows signing.

## Consequences

- No new hosting and no `.deb` files committed to git.
- Only the latest version is installable through apt; older ones stay on the releases page.
- Clients can see a hash mismatch for a few seconds while a release's assets upload; `apt update`
  succeeds on retry.
- The signing key has to be backed up, renewed before it expires, and rotated if compromised.
