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

- The affected component (desktop app, `gitvault` CLI, Node library, Vite plugin, vault format).
- The GitVault version and your operating system.
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

- Weaknesses in the vault format or cryptography (see `docs/vault-format.md`): key handling,
  password or recovery-key unwrapping, nonce use, tampering that goes undetected.
- Secret values, passwords, tokens or recovery keys being logged, cached, written to disk in
  plaintext, or exposed through URLs or command-line arguments.
- Flaws in the desktop app, CLI, Node library or Vite plugin that expose vault contents or
  let another local process or website read them.
- Problems in the installers or the updater that allow tampered builds to be installed.

Out of scope:

- Attacks that need an already-compromised machine (malware, keyloggers, root access) or
  physical access to an unlocked session.
- Weak master passwords chosen by the user.
- Vulnerabilities in third-party services such as GitHub, or in dependencies with no
  demonstrated impact on GitVault.
- Denial of service through resource exhaustion on your own machine.

## Safe harbour

We will not pursue legal action against good-faith research that follows this policy, stays
within your own vaults and accounts, and avoids accessing other people's data.
