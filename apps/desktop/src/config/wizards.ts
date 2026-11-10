/** Steps of the multi-page flows. Each step is its own route (`/vaults/new/$step`). */

export const CREATE_VAULT_STEPS = ['name', 'repository', 'password', 'recovery', 'done'] as const
export type CreateVaultStep = (typeof CREATE_VAULT_STEPS)[number]

export const ADD_SECRET_STEPS = ['type', 'details', 'value', 'expiry', 'review', 'saved'] as const
export type AddSecretStep = (typeof ADD_SECRET_STEPS)[number]

export const CREATE_VAULT_LABELS: Record<CreateVaultStep, string> = {
  name: 'Name',
  repository: 'Repository',
  password: 'Master password',
  recovery: 'Recovery key',
  done: 'Done',
}

export const ADD_SECRET_LABELS: Record<AddSecretStep, string> = {
  type: 'Type',
  details: 'Details',
  value: 'Value',
  expiry: 'Expiry & notes',
  review: 'Review',
  saved: 'Saved',
}

export interface PanelContent {
  headline: string
  description: string
}

export const CREATE_VAULT_PANELS: Record<CreateVaultStep, PanelContent> = {
  name: {
    headline: 'Name your vault',
    description: 'Give your vault a name your team will recognize. You can change it later.',
  },
  repository: {
    headline: 'Choose where it lives',
    description: 'Vaults are stored as private repositories in your GitHub account.',
  },
  password: {
    headline: 'Protect your vault',
    description: 'Your master password encrypts everything locally. GitHub never sees it.',
  },
  recovery: {
    headline: 'One last safety net',
    description:
      'Keep your recovery key offline. It restores your vault if you forget your master password.',
  },
  done: {
    headline: 'You are all set',
    description: 'Your vault is encrypted and synced to GitHub. Start adding secrets.',
  },
}

export const ADD_SECRET_PANELS: Record<AddSecretStep, PanelContent> = {
  type: {
    headline: 'What are you adding?',
    description: 'Pick a type so GitVault can tailor the fields and masking.',
  },
  details: {
    headline: 'Describe it',
    description: 'A clear name and environment make secrets easy to find and audit.',
  },
  value: {
    headline: 'Add the value',
    description: 'Paste it in or generate one. It is encrypted before it is saved.',
  },
  expiry: {
    headline: 'Plan for rotation',
    description: 'Expiry dates and reminders keep long-lived keys from going stale.',
  },
  review: {
    headline: 'Review and save',
    description: 'Double-check the details. Saving pushes an encrypted commit to GitHub.',
  },
  saved: {
    headline: 'Secret saved',
    description: 'It is encrypted, versioned and synced to GitHub.',
  },
}

export const UNLOCK_PANEL: PanelContent = {
  headline: 'Welcome back',
  description: 'Your vault is locked. Enter your master password to unlock it.',
}

export function isStep<T extends readonly string[]>(steps: T, value: string): value is T[number] {
  return steps.includes(value)
}

/** Panel text for a wizard URL path such as `/vaults/new/name` or `/vaults/abc/secrets/new/value`. */
export function getWizardPanel(pathname: string): PanelContent {
  const segments = pathname.split('/').filter(Boolean)
  const step = segments.at(-1) ?? ''
  if (segments.includes('unlock')) return UNLOCK_PANEL
  if (segments.includes('secrets') && isStep(ADD_SECRET_STEPS, step)) return ADD_SECRET_PANELS[step]
  if (isStep(CREATE_VAULT_STEPS, step)) return CREATE_VAULT_PANELS[step]
  return CREATE_VAULT_PANELS.name
}
