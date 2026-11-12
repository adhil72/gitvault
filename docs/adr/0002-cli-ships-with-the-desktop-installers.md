# ADR 0002: The CLI ships inside the desktop installers

## Status

Accepted

## Context

The Node library (`@adhil28/gitvault`) and terminal users need the `gitvault` binary on `PATH`.
Installing the desktop app did not install it, so users had to build or download it separately.

## Decision

- The `gitvault` binary is bundled as a Tauri sidecar (`bundle.externalBin`) and installed next to
  the app. `.deb` installs it to `/usr/bin/gitvault`, which is already on `PATH`. The Windows NSIS
  installer adds its install folder to the user `PATH` (`installer/hooks.nsh`) and removes it on
  uninstall.
- The sidecar is declared in `src-tauri/tauri.bundle.conf.json`, merged only for release builds
  (`pnpm bundle`, and the release workflow). Tauri refuses to build when a sidecar file is missing,
  so keeping it out of `tauri.conf.json` leaves `cargo clippy`, `tauri dev` and CI untouched.
- `scripts/prepare-sidecar.mjs` builds the CLI and copies it to
  `src-tauri/binaries/gitvault-<target-triple>`. That folder is git-ignored.

## Consequences

- The CLI and the app share a version and update together through the app updater.
- Installers grow by the size of the CLI binary.
- The CLI is a copy: a separately installed `gitvault` earlier on `PATH` still wins.
