import js from '@eslint/js'
import reactHooks from 'eslint-plugin-react-hooks'
import globals from 'globals'
import tseslint from 'typescript-eslint'

/**
 * Import boundaries (docs/architecture.md section 9.5):
 * routes -> components/pages -> components/general -> components/ui -> lib
 * `api/` is only imported by `hooks/`.
 */
const restrict = (patterns) => ({
  'no-restricted-imports': ['error', { patterns }],
})

const noApiFromUi = { group: ['@/api', '@/api/*'], message: 'Only hooks/ may import api/.' }

export default tseslint.config(
  { ignores: ['dist', 'src/routeTree.gen.ts', 'src/types/generated', 'src-tauri'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: { globals: globals.browser },
    plugins: { 'react-hooks': reactHooks },
    rules: {
      ...reactHooks.configs.recommended.rules,
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-imports': 'error',
      // Size and shape limits (docs/architecture.md section 10.2)
      'max-lines': ['error', { max: 300, skipBlankLines: true, skipComments: true }],
      'max-lines-per-function': ['error', { max: 150, skipBlankLines: true, skipComments: true }],
      'max-params': ['error', 3],
      'max-depth': ['error', 3],
      complexity: ['error', 8],
    },
  },
  // Plain functions (non-component files) stay small; hooks may be a little longer.
  {
    files: ['src/{lib,api,stores,schemas,config}/**/*.ts'],
    rules: {
      'max-lines-per-function': ['error', { max: 30, skipBlankLines: true, skipComments: true }],
    },
  },
  {
    files: ['src/hooks/**/*.ts'],
    rules: {
      'max-lines-per-function': ['error', { max: 60, skipBlankLines: true, skipComments: true }],
    },
  },
  // The browser-only mock backend is development tooling, exempt from size rules.
  {
    files: ['src/api/mock/**'],
    rules: {
      'max-lines': 'off',
      'max-lines-per-function': 'off',
      complexity: 'off',
      'max-params': 'off',
    },
  },
  // React components branch a lot in JSX; allow a little more than plain functions.
  {
    files: ['src/components/**/*.tsx', 'src/routes/**/*.tsx'],
    rules: { complexity: ['error', 12] },
  },
  // Route files only wire things together.
  {
    files: ['src/routes/**/*.tsx'],
    rules: {
      'max-lines': ['error', { max: 40, skipBlankLines: true, skipComments: true }],
      ...restrict([
        noApiFromUi,
        {
          regex: '^@/components/ui/(?!sonner)',
          message: 'Routes import page components (and layouts) only.',
        },
        {
          regex: '^@/components/general/(?!layouts/|page-placeholder|theme-sync)',
          message: 'Routes import page components (and layouts) only.',
        },
      ]),
    },
  },
  {
    files: ['src/components/pages/**'],
    rules: restrict([
      noApiFromUi,
      { group: ['@/routes/*'], message: 'Pages must not import routes.' },
    ]),
  },
  {
    files: ['src/components/general/**'],
    rules: restrict([
      noApiFromUi,
      {
        group: ['@/routes/*', '@/components/pages/*'],
        message: 'general/ must not import pages or routes.',
      },
    ]),
  },
  // shadcn primitives are generated; they may import only lib/.
  {
    files: ['src/components/ui/**'],
    rules: {
      'max-lines-per-function': 'off',
      complexity: 'off',
      ...restrict([
        noApiFromUi,
        {
          group: [
            '@/components/general/*',
            '@/components/pages/*',
            '@/hooks/*',
            '@/stores/*',
            '@/routes/*',
          ],
          message: 'ui/ may import only lib/.',
        },
      ]),
    },
  },
  {
    files: ['src/hooks/**'],
    rules: restrict([
      { group: ['@/components/*', '@/routes/*'], message: 'hooks/ must not import UI.' },
    ]),
  },
)
