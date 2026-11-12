/** Steps of the multi-page flows. Each step is its own route (`/vaults/new/$step`). */

export const CREATE_VAULT_STEPS = ['name', 'repository', 'password', 'recovery'] as const
export type CreateVaultStep = (typeof CREATE_VAULT_STEPS)[number]

export const ADD_SECRET_STEPS = ['details', 'value', 'expiry', 'review', 'saved'] as const
export type AddSecretStep = (typeof ADD_SECRET_STEPS)[number]

export const CREATE_VAULT_LABELS: Record<CreateVaultStep, string> = {
  name: 'Name',
  repository: 'Repository',
  password: 'Master password',
  recovery: 'Recovery key',
}

export const ADD_SECRET_LABELS: Record<AddSecretStep, string> = {
  details: 'Details',
  value: 'Value',
  expiry: 'Expiry & notes',
  review: 'Review',
  saved: 'Saved',
}

export function isStep<T extends readonly string[]>(steps: T, value: string): value is T[number] {
  return steps.includes(value)
}
