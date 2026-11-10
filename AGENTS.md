# AGENTS.md — rules for AI coding tools working on GitVault

Read `docs/architecture.md` (§2 overview, §4 layout, §9 UI, §10 coding rules) before changing code.

## Non-negotiables

1. **Business logic lives only in `crates/gitvault-core`.** `src-tauri`, `gitvault-cli` and React components only translate input/output and call a use case.
2. New capability order: **core use case -> CLI command -> Tauri command -> UI**. Every capability must exist in both CLI and UI (or be listed as `ui_only` / `cli_only` with a reason).
3. **Never log, print, commit, cache or put in a URL/argument any secret value, token, password or recovery key.**
4. Respect the dependency direction: `crypto <- core <- infra <- cli / desktop`. Core has no I/O.

## Frontend structure (apps/desktop/src)

- `routes/` routing only, thin files (<= 40 lines): `createFileRoute`, `validateSearch`, `beforeLoad`, `loader`, `component: SomePage`.
- `components/ui/` shadcn primitives (generated with `pnpm dlx shadcn@latest add <name>`, then customized; log changes in `components/ui/CHANGES.md`). Imports only `lib/`.
- `components/general/` reusable composed components used by 2+ pages (selectors, fields, layouts).
- `components/pages/<page>/` page components; the ONLY components routes import. A page never imports another page.
- `api/` typed wrappers over Tauri commands (only `hooks/` imports it). `hooks/` TanStack Query hooks. `stores/` Zustand (in-memory, never secrets). `schemas/` zod. `lib/` pure utilities.
- Files are kebab-case, one component per file, named exports only.

## Code rules (enforced by lint/CI)

- TypeScript: functions <= 30 lines (hooks <= 60, React components <= 150), <= 3 parameters, nesting <= 3, complexity <= 8 (components <= 12), files <= 300 lines, route files <= 40 lines.
- Rust: functions <= 40 lines, <= 5 parameters plus `self`, cognitive complexity <= 12.
- No `unwrap()`/`expect()`/`panic!` in non-test Rust; no `any` in TypeScript.
- Parse, don't validate (newtypes like `SecretName`). Errors are typed with stable codes (`ErrorCode`).
- Comments explain _why_. Name things by what they are; no `utils`/`helpers` dumping files.
- SOLID, DRY (abstract on the third repetition), KISS, YAGNI.

## Where things are

- Services: `crates/gitvault-core/src/application/*`; ports: `.../ports`; adapters: `crates/gitvault-infra`.
- New DTO or command? Derive it in `crates/gitvault-core/src/dto`, run `pnpm gen:types`, add the Tauri command in `apps/desktop/src-tauri/src/commands`, the CLI command in `crates/gitvault-cli/src/commands`, the `api/` wrapper and a `hooks/` hook.
- The browser build uses `src/api/mock` (press "Continue with GitHub CLI" to see sample data).

## Before finishing a task

```bash
pnpm check:all    # frontend (format, lint, types, tests) + Rust (fmt, clippy, tests)
```

Update `docs/architecture.md` / add an ADR in `docs/adr/` when changing architecture.
