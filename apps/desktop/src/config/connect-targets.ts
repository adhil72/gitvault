export interface ConnectStep {
  title: string
  /** Optional sentence under the title. */
  hint?: string
  language: 'bash' | 'javascript'
  code: string
}

export interface ConnectTarget {
  id: string
  label: string
  description: string
  steps: readonly ConnectStep[]
}

/**
 * Ways to connect an app to a vault. The Connect dialog renders whatever is listed here, so
 * supporting another runtime or tool means appending one entry: no UI change. Snippets are static
 * text and must never contain a secret value, token or key.
 */
export const CONNECT_TARGETS: readonly ConnectTarget[] = [
  {
    id: 'nodejs',
    label: 'Node.js',
    description: 'Load this vault into process.env at startup. No .env file in the project.',
    steps: [
      {
        title: 'Install the package',
        language: 'bash',
        code: 'npm install @adhil28/gitvault',
      },
      {
        title: 'Trust this device once',
        hint: 'The gitvault CLI is installed with the desktop app. Sign in to GitHub first (gh auth login).',
        language: 'bash',
        code: 'gitvault device trust --name laptop',
      },
      {
        title: 'Load secrets on the first line of your app',
        hint: 'Uses GITVAULT_ENV, then NODE_ENV, to pick the environment.',
        language: 'javascript',
        code: "import '@adhil28/gitvault/config'",
      },
      {
        title: 'Or load them explicitly',
        hint: 'Returns the names that were loaded, never their values.',
        language: 'javascript',
        code: "import { config } from '@adhil28/gitvault'\n\nconst { environment, loaded } = config('production')",
      },
    ],
  },
]
