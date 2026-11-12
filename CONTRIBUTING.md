# Contributing to GitVault

Thanks for helping. GitVault is MIT licensed, and by contributing you agree your work is released under the same license.

## Before you start

- Read [AGENTS.md](AGENTS.md) and [docs/architecture.md](docs/architecture.md) (§2 overview, §4 layout, §9 UI, §10 coding rules).
- For anything larger than a small fix, open an issue first so we can agree on the approach.
- **Security problems:** do not open a public issue. Follow [SECURITY.md](SECURITY.md).

## Ground rules

1. Business logic lives only in `crates/gitvault-core`. The CLI, the Tauri shell and React components translate input and output.
2. New capability order: core use case, CLI command, Tauri command, UI. Every capability exists in both CLI and UI, or is listed as `ui_only` / `cli_only` with a reason.
3. Never log, print, commit, cache or put in a URL or argument any secret value, token, password or recovery key. This includes tests, fixtures and screenshots.
4. Keep the dependency direction: `crypto <- core <- infra <- cli / desktop`.

## Development

```bash
pnpm install
pnpm check:all    # frontend (format, lint, types, tests) + Rust (fmt, clippy, tests)
pnpm gen:types    # after changing Rust DTOs
pnpm gen:docs     # after changing CLI commands
```

The browser build uses mock data (`apps/desktop/src/api/mock`), so you can work on the UI without a GitHub account.

## Pull requests

- Branch from `main`, keep the PR focused, and make sure `pnpm check:all` passes.
- Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `chore:`).
- Update docs, and add an ADR in `docs/adr/` when changing architecture.
- Changes to cryptography or the vault format need extra care: explain the reasoning and add tests.
