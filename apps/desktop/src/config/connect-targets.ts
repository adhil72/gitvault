import type { IconType } from 'react-icons'
import { SiNodedotjs, SiVite } from 'react-icons/si'

export interface ConnectStep {
  title: string
  /** Optional sentence under the title. */
  hint?: string
  /** File the snippet belongs in, or the shell for commands. Shown above the code. */
  filename: string
  language: 'bash' | 'javascript' | 'typescript'
  code: string
}

export type DocCode = Pick<ConnectStep, 'filename' | 'language' | 'code'>

/** One piece of the long-form guide shown on the integration docs page. */
export type DocBlock =
  | { kind: 'text'; text: string }
  | { kind: 'list'; items: readonly string[] }
  | { kind: 'note'; text: string }
  | ({ kind: 'code' } & DocCode)

export interface ConnectTarget {
  id: string
  label: string
  /** Short tagline under the label in the framework switcher. */
  tagline: string
  icon: IconType
  description: string
  steps: readonly ConnectStep[]
  /** Long-form guide for the docs page, shown while this target is selected. */
  guide: readonly DocBlock[]
}

const INSTALL_STEP: ConnectStep = {
  title: 'Install the package',
  filename: 'terminal',
  language: 'bash',
  code: 'npm install @adhil28/gitvault',
}

const TRUST_STEP: ConnectStep = {
  title: 'Trust this device once',
  hint: 'The gitvault CLI is installed with the desktop app. Sign in to GitHub first (gh auth login).',
  filename: 'terminal',
  language: 'bash',
  code: 'gitvault device trust --name laptop',
}

/**
 * Ways to connect an app to a vault. The Connect dialog renders whatever is listed here, so
 * supporting another runtime or tool (Bootstrap, Flutter, ...) means appending one entry with its
 * own steps and guide: no UI change. Steps are per target on purpose, since not every target uses
 * npm. Snippets are static
 * text and must never contain a secret value, token or key.
 */
export const CONNECT_TARGETS: readonly ConnectTarget[] = [
  {
    id: 'nodejs',
    label: 'Node.js',
    tagline: 'Servers, scripts, CLIs',
    icon: SiNodedotjs,
    description: 'Load this vault into process.env at startup. No .env file in the project.',
    steps: [
      INSTALL_STEP,
      TRUST_STEP,
      {
        title: 'Load secrets on the first line of your app',
        hint: 'Uses GITVAULT_ENV, then NODE_ENV, to pick the environment.',
        filename: 'index.js',
        language: 'javascript',
        code: "import '@adhil28/gitvault/config'",
      },
      {
        title: 'Or load them explicitly',
        hint: 'Returns the names that were loaded, never their values.',
        filename: 'index.js',
        language: 'javascript',
        code: "import { config } from '@adhil28/gitvault'\n\nconst { environment, loaded } = config('production')",
      },
    ],
    guide: [
      {
        kind: 'text',
        text: 'Import the config entry on the first line. It picks the environment from GITVAULT_ENV, then NODE_ENV.',
      },
      {
        kind: 'code',
        filename: 'index.js',
        language: 'javascript',
        code: "import '@adhil28/gitvault/config'",
      },
      { kind: 'text', text: 'Or call it explicitly to choose the environment and options:' },
      {
        kind: 'code',
        filename: 'index.js',
        language: 'javascript',
        code: "import { config } from '@adhil28/gitvault'\n\nconst { environment, loaded } = config('production', {\n  vault: 'owner/repo', // default: GITVAULT_VAULT, then the selected vault\n  override: false, // true replaces variables that are already set\n  bin: 'gitvault', // default: GITVAULT_BIN, then gitvault on PATH\n})",
      },
      { kind: 'text', text: 'CommonJS projects use require:' },
      {
        kind: 'code',
        filename: 'index.cjs',
        language: 'javascript',
        code: "require('@adhil28/gitvault/config')",
      },
    ],
  },
  {
    id: 'vite',
    label: 'Vite',
    tagline: 'React, Vue, Svelte, ...',
    icon: SiVite,
    description:
      'Load this vault before Vite reads its env. No .env file; works with React, Vue, Svelte and any other Vite app.',
    steps: [
      INSTALL_STEP,
      TRUST_STEP,
      {
        title: 'Add the plugin',
        hint: 'The environment follows the Vite mode: development for vite, production for vite build.',
        filename: 'vite.config.ts',
        language: 'typescript',
        code: "import { defineConfig } from 'vite'\nimport { gitvault } from '@adhil28/gitvault/vite'\n\nexport default defineConfig({\n  plugins: [gitvault()],\n})",
      },
      {
        title: 'Use the values in your code',
        hint: 'Only names starting with VITE_ reach the browser. Everything else stays server-side.',
        filename: 'src/main.ts',
        language: 'typescript',
        code: 'const apiUrl = import.meta.env.VITE_API_URL',
      },
    ],
    guide: [
      {
        kind: 'text',
        text: 'The plugin loads the vault before Vite reads its env. It works with React, Vue, Svelte and any other Vite 5+ app.',
      },
      {
        kind: 'code',
        filename: 'vite.config.ts',
        language: 'typescript',
        code: "import { defineConfig } from 'vite'\nimport { gitvault } from '@adhil28/gitvault/vite'\n\nexport default defineConfig({\n  plugins: [gitvault()], // or gitvault({ environment: 'staging' })\n})",
      },
      {
        kind: 'list',
        items: [
          'The environment is options.environment, then GITVAULT_ENV, then the Vite mode: vite is development, vite build is production, --mode staging is staging.',
          'Vite exposes only variables starting with VITE_ (the envPrefix) to browser code as import.meta.env. Everything else stays in process.env for the dev server and build tooling.',
          'The plugin runs after vite.config.ts is evaluated, so the config file cannot read vault values. Add import "@adhil28/gitvault/config" at the top of the config file if it must.',
          'It logs only how many variables were loaded, never names or values.',
        ],
      },
      {
        kind: 'code',
        filename: 'src/main.ts',
        language: 'typescript',
        code: 'const apiUrl = import.meta.env.VITE_API_URL',
      },
      {
        kind: 'note',
        text: 'Anything behind the VITE_ prefix ends up in the browser bundle. Never name a real secret VITE_something.',
      },
    ],
  },
]
