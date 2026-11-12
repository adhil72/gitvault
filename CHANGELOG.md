# Changelog

All notable changes to GitVault are listed here, newest first. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/). The same history is published at
https://adhil72.github.io/gitvault-website/changelog/.

## [1.0.0] - 2026-11-12

### Changed

- **First stable release.** GitVault is now 1.0.0. The sign-in screen shows the installed app
  version instead of a fixed number.

## [0.6.1] - 2026-11-12

### Added

- **Signed apt repository.** Ubuntu and Debian users can install and update GitVault with `apt`.
  Each release publishes a GPG-signed repository; see `docs/apt-repository.md` for the install
  steps and the key fingerprint.
- **Security policy.** `SECURITY.md` explains how to report a vulnerability privately.

## [0.6.0] - 2026-11-12

### Added

- **Vite plugin.** `@adhil28/gitvault/vite` loads a GitVault environment before Vite reads its env,
  so Vite projects need no `.env` file. The environment defaults to `GITVAULT_ENV`, then the Vite
  mode; only `VITE_` variables reach client code.

## [0.5.0] - 2026-11-12

### Added

- **The CLI comes with the desktop app.** The Windows and Ubuntu installers now include the
  `gitvault` command and put it on `PATH` (Windows: the user `PATH`, removed again on uninstall;
  Ubuntu: `/usr/bin/gitvault`), so the Node library and terminals work right after installing.
- **Connect button.** The header opens a dialog with copyable steps to load a vault into a Node.js
  app. More runtimes can be added by listing them in `config/connect-targets.ts`.
- **Switch warnings.** Switching vault or GitHub account asks for confirmation first.

## [0.4.1] - 2026-11-12

### Changed

- **Node library tests in CI.** The `@adhil28/gitvault` tests run against the real CLI on pull
  requests. The package is published to npm by hand rather than by the release workflow.
- **Node library package.** Ships a `LICENSE` file, type definitions for `@adhil28/gitvault/config`,
  keywords and a homepage link.

## [0.4.0] - 2026-11-11

### Added

- **Collaborators.** Invite GitHub users to a vault's repository, read-only or able to change
  secrets, list them, and remove them or cancel a pending invitation. Available in the desktop app
  under Settings → GitHub & Sync, and as `gitvault collaborator list | invite | remove`. Inviting
  grants repository access only: they still need the master password to open the vault, which you
  share separately.

## [0.3.0] - 2026-11-11

A visual refresh of the desktop app, a collapsible sidebar, and a security hardening of how vault
and secret ids are read from a cloned repository.

### Added

- **Collapsible sidebar.** A button in the header (and `Ctrl`/`Cmd`+`B`) folds the vault sidebar
  down to an icon rail; hovering a rail icon shows its label. The sidebar collapses by itself while
  Settings is open, to give the settings pane room, and returns to how you had it when you leave.
  Toggling it by hand always wins over the automatic behaviour.
- **Settings tab icons** for General, Security, GitHub & Sync and Appearance.
- **Grouped settings.** Settings are organised into labelled groups (Session, Protection, Sync,
  About, Account, Master password, Recovery key, Conflicts, Trusted devices), with explanations
  moved below each card so rows stay uncluttered.
- **Motion that respects you.** Pages, empty states and settings tabs rise in softly, and buttons
  and navigation links give a small press response. All of it is switched off when the system
  "reduce motion" preference is on.

### Changed

- **Whole vault cards are clickable.** Open a vault from the start screen by clicking anywhere on
  its card instead of finding the small Open button. Vaults that exist on GitHub but are not
  connected yet keep their Connect button.
- **Unlock dialog redesigned.** Centered lock icon, a full-width "Unlock vault" button, and a
  clearer "Trust this device" option with a one-line explanation.
- **Calmer sidebar.** Zero counts are no longer shown next to "All secrets", "Favorites" and
  "Expiring soon". The active item is marked with a highlight and an accent icon instead of a bar.
- **Darker, more consistent dark theme.** Background, card, popover and sidebar surfaces were
  re-tuned so layers read clearly against each other.
- **Lighter chrome.** Hairline borders, a shorter header (56 px), a softer search field, and larger
  page titles with tighter letter spacing.
- **Trusted devices** are laid out as one grouped list, with each device's trust date as a row.
- **Buttons** give press feedback by scaling slightly on pointer-down and only animate colour,
  transform and shadow (recorded in `components/ui/CHANGES.md`).
- **Empty states** drop the bordered tile for a softly glowing icon.

### Security

- **Ids are validated when read from disk, not only when created.** `VaultId` and `SecretId` now
  validate on deserialization, so a cloned repository whose vault header contains an id with path
  separators (for example `../../escape`) is rejected instead of being used to build file paths.
  Covered by a new test in `gitvault-core`.

## [0.2.2] - 2026-11-11

### Fixed

- **Linux installs can now update.** The `.deb` is listed in the update feed (`latest.json`) next
  to the Windows installer, so Ubuntu installs get update notices as well.

### Changed

- Releases are produced from a single push to `main`: a commit whose version has no tag yet is
  tagged and released by the Release workflow. CI runs only on pull requests, so `main` no longer
  runs the same checks twice.

## [0.2.1] - 2026-11-11

### Added

- **Choose the GitHub account when naming a vault.** The new-vault wizard lists your GitHub CLI
  accounts and lets you pick which one owns the vault; the field is hidden when `gh` knows no
  accounts.

### Changed

- **The new vault opens as soon as setup finishes.** The separate "done" step was removed; saving
  the recovery key takes you straight into the vault.

## [0.2.0] - 2026-11-11

### Added

- **Pick a vault on start.** The app opens on your list of vaults, built from a device-local index
  of vault names, so nothing needs to be decrypted to browse them.
- **Switch GitHub CLI accounts** from the account menu in the header.
- **Unlock dialog.** Actions that need the key (reveal, copy, edit, rotate and similar) ask for the
  master password in a dialog at the moment you need it.

### Changed

- **Vaults open locked.** Opening a vault no longer asks for the master password up front, and a
  notice in the vault explains that it is locked. The separate unlock page was removed.

## [0.1.3] - 2026-11-11

### Fixed

- Release build: allow esbuild's install script so the Node library's `tsup` build installs under
  pnpm 12. Needed to publish `@adhil72/gitvault` to GitHub Packages.

## [0.1.2] - 2026-11-11

### Changed

- **The Node.js library is written in TypeScript**, ships type definitions, and supports both
  `import` and `require`.
- **Published on GitHub Packages** as `@adhil72/gitvault`.
- Releases are published in the public website repository and mirrored in this one.

## [0.1.1] - 2026-11-10

### Added

- **Signed auto-updater.** The Windows app checks for signed updates, installs them from Settings,
  and shows a quiet notice at startup.
- **`SHA256SUMS`** is published with every release so downloads can be verified with
  `sha256sum -c SHA256SUMS --ignore-missing`.

## [0.1.0] - 2026-11-10

First public release.

### Added

- Create, connect, unlock and lock encrypted vaults stored in a private GitHub repository.
- Add, edit, rotate and delete secrets, with environments, tags, favorites and expiry reminders.
- Master password with Argon2id key derivation, AES-256-GCM encryption and a one-time recovery key.
- Every change is a git commit, with version history and restore for any secret.
- Sync with GitHub, with side-by-side conflict resolution when two devices edit the same secret.
- Audit log of reveals and changes.
- Desktop app for Windows and Ubuntu, with a password, passphrase and PIN generator.
- `gitvault` CLI: `run` and `inject` secrets without printing them, import and export `.env`
  files, JSON output and shell completions.
- Node.js package: `@adhil72/gitvault/config` loads secrets into `process.env`.
- MCP server (`gitvault mcp`) so AI tools can use secrets without seeing their values.

[0.3.0]: https://github.com/adhil72/gitvault/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/adhil72/gitvault/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/adhil72/gitvault/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/adhil72/gitvault/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/adhil72/gitvault/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/adhil72/gitvault/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/adhil72/gitvault/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/adhil72/gitvault/releases/tag/v0.1.0
