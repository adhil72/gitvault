# ADR 0001: Vaults open locked; names come from a device-local index

## Status

Accepted

## Context

Opening the app used to jump into the last vault and redirect to an unlock page until the master
password was typed. We want: pick a vault first (and switch GitHub CLI accounts), open it without a
password, see what is in it, and only type the password (in a dialog) when something needs the
vault key (reveal, copy, edit, add, import, delete).

Secret names, environments and tags are inside the encrypted file (see `docs/vault-format.md`), so
they cannot be listed without the key.

## Decision

- **No vault format change.** Secret files and the GitHub repository stay fully encrypted.
- Every time the vault is read while unlocked, core writes an index of names and metadata (never
  values or notes) to this device's private key-value store (`catalog:<vault-id>`). It is never
  committed, synced or uploaded.
- While locked, `secrets.list`, `secrets.facets` and `secrets.get` are answered from that index.
  Everything else (reveal, copy, add, update, delete, history, import, export) still needs the key
  and returns `E_LOCKED`.
- A vault this device never unlocked has no index: the UI shows "locked" and asks for the password.
- Forgetting a vault deletes its index.
- The UI wraps key-needing actions in `useRequireUnlock`, which opens `UnlockDialog` and runs the
  action after a successful unlock.
- `GhCli` gains `accounts()` and `token_for(login)`; the app signs in with the active `gh` account
  on start and lets the user switch accounts from the header (`gh auth token --user`, the CLI's
  own active account is untouched).

## Consequences

- Secret names are stored in plaintext in the device's state file (mode 0600), next to the GitHub
  token that is already stored there. Anyone who can read that file can read names but not values.
- Changes made on another device show up in the locked list only after this device unlocks again.
