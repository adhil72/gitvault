# Contributing to GitVault

Thanks for helping. GitVault is MIT licensed, and by contributing you agree your work is released under the same license.

## Ground rules

1. Keep the CLI purely driven by `git` and `gh` CLIs — do not add internal GitHub API clients, OAuth flows, or token stores.
2. Keep the zero-`.env` guarantee: `init` and `load` must never create plaintext `.env` files on disk.
3. Never log, print, commit, or expose secret values, master passwords, or recovery keys.
4. Respect [clippy.toml](clippy.toml) complexity and function-length limits (`<= 40` lines, cognitive complexity `<= 12`, `#![forbid(unsafe_code)]`, no `unwrap`/`expect`/`panic` outside tests).

## Verification

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Pull requests

- Keep PRs focused and add scenario tests under `tests/` for new workflows or regressions.
- Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `chore:`).
