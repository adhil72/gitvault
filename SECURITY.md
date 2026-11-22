# Security Policy

GitVault stores secrets, so we treat security reports as the highest priority.

## Supported versions

Only the latest release receives security fixes. Please update before reporting, and check
that the issue still reproduces.

## Reporting a vulnerability

**Do not open a public issue, pull request or discussion for a security problem.**

Report it privately through GitHub:
[Report a vulnerability](https://github.com/adhil72/gitvault/security/advisories/new)

Please include:

- The affected component (`gitvault` CLI, crypto/vault format, `git`/`gh` driver, shell integration).
- The GitVault version (`gitvault --version`) and your operating system.
- Steps to reproduce, and the impact you expect.

Never include real secrets, master passwords or recovery keys in a report. Use throwaway
test vaults.

### What to expect

- We acknowledge a report within 5 business days.
- We aim to confirm the issue and share an assessment within 14 days.
- We fix confirmed issues as quickly as their severity allows, then publish an advisory and
  credit you if you want to be named.

Please give us reasonable time to release a fix before disclosing the issue publicly.

## Scope

In scope:

- Weaknesses in the v2 vault format or cryptography: Argon2id key derivation, AES-256-GCM
  encryption, AAD binding (`gitvault:v2:<project>:<environment>`), recovery-key unwrapping,
  nonce reuse, or tampering that goes undetected.
- Secret values, passwords, or recovery keys being logged, written to `.env` or plaintext
  files on disk, or exposed via shell injection during `gitvault load --export`.
- Incorrect filesystem permissions on `~/.local/share/gitvault/store` (`0700`) or
  `~/.local/share/gitvault/session.key` (`0600`).

Out of scope:

- Attacks that require an already-compromised machine (malware, keyloggers, root access) or
  physical access to an unlocked `session.key`.
- Weak master passwords chosen by the user.
- Vulnerabilities in `git`, `gh`, or GitHub itself.

## Safe harbour

We will not pursue legal action against good-faith research that follows this policy, stays
within your own vaults and accounts, and avoids accessing other people's data.
