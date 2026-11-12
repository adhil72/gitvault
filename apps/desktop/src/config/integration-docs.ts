import type { DocBlock } from '@/config/connect-targets'

export interface DocSection {
  id: string
  title: string
  blocks: readonly DocBlock[]
}

/**
 * Content of the integration docs page. Static text only: it must never contain a secret value,
 * token or key (the device key and token below are variable names, not values).
 */
export const INTRO_DOCS: readonly DocSection[] = [
  {
    id: 'how-it-works',
    title: 'How it works',
    blocks: [
      {
        kind: 'text',
        text: 'Your app never holds the vault password. At startup the GitVault library asks the gitvault CLI (installed with the desktop app) for one environment. The CLI checks your GitHub access, decrypts on this device and hands the values to your process as environment variables. Nothing is written to disk and no .env file is needed.',
      },
      {
        kind: 'list',
        items: [
          'Secrets of the chosen environment plus shared ones (no environment) are loaded. The environment value wins on a clash.',
          'Variables already set in the shell win over the vault, unless you pass override: true.',
          'Loading is synchronous, so it works on the first line of an app.',
          'The library returns names that were loaded, never values.',
        ],
      },
    ],
  },
  {
    id: 'setup',
    title: 'One-time setup',
    blocks: [
      { kind: 'text', text: 'Install the package in your project:' },
      {
        kind: 'code',
        filename: 'terminal',
        language: 'bash',
        code: 'npm install @adhil28/gitvault',
      },
      {
        kind: 'text',
        text: 'Sign in to GitHub, then trust this device once so it unlocks without a password. Alternatively tick the trust checkbox on the unlock page of the app.',
      },
      {
        kind: 'code',
        filename: 'terminal',
        language: 'bash',
        code: 'gh auth login\ngitvault device trust --name laptop',
      },
      {
        kind: 'text',
        text: 'To limit unlocking to one project, run this inside the project folder instead. The device then unlocks by itself only there and in its subfolders, and only for the listed environments:',
      },
      {
        kind: 'code',
        filename: 'terminal',
        language: 'bash',
        code: 'gitvault trust add . --env development',
      },
    ],
  },
]

/** Shown after the framework guide. Framework guides live on each `ConnectTarget`. */
export const OUTRO_DOCS: readonly DocSection[] = [
  {
    id: 'servers-ci',
    title: 'Servers and CI',
    blocks: [
      {
        kind: 'text',
        text: 'A server has no GitHub CLI session. Export a device key once, then give the process a read-only GitHub token for the vault repository.',
      },
      {
        kind: 'code',
        filename: 'terminal',
        language: 'bash',
        code: 'gitvault device trust --name prod-server --export',
      },
      {
        kind: 'list',
        items: [
          'Set the printed key as GITVAULT_DEVICE_KEY. It is shown once.',
          'Set GITVAULT_TOKEN to a read-only GitHub token for the vault repo.',
          'GITVAULT_PASSWORD also works instead of a device key.',
          'Revoke a device any time with gitvault device revoke <id>.',
        ],
      },
      {
        kind: 'note',
        text: 'Store these two values in your CI provider or host secret store, never in the repository.',
      },
    ],
  },
  {
    id: 'troubleshooting',
    title: 'Troubleshooting',
    blocks: [
      {
        kind: 'text',
        text: 'Failures throw a GitVaultError with a stable code:',
      },
      {
        kind: 'list',
        items: [
          'E_LOCKED: this device is not trusted for the vault or folder. Run gitvault device trust, or gitvault trust add inside the project.',
          'E_NOT_AUTHENTICATED: sign in with gh auth login (or set GITVAULT_TOKEN on a server).',
          'E_PERMISSION_DENIED: your GitHub account cannot read the vault repository.',
          'E_NOT_INSTALLED: the gitvault CLI was not found. Install the desktop app, or set GITVAULT_BIN to its path.',
          'E_NOT_FOUND: the vault or environment does not exist. Check the name and the selected vault.',
        ],
      },
    ],
  },
]
