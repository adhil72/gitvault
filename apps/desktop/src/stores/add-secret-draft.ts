import { create } from 'zustand'

import type { SecretKind } from '@/types'

/** In-memory draft of the add-secret wizard. Cleared on finish, cancel and lock. Never persisted. */
interface AddSecretDraft {
  kind: SecretKind
  name: string
  environment: string
  tags: string
  value: string
  expires: string
  remindDays: string
  notes: string
  savedName: string
  savedId: string
  update: (patch: Partial<Omit<AddSecretDraft, 'update' | 'reset'>>) => void
  reset: () => void
}

const initial = {
  kind: 'api_key' as SecretKind,
  name: '',
  environment: '',
  tags: '',
  value: '',
  expires: '',
  remindDays: '7',
  notes: '',
  savedName: '',
  savedId: '',
}

export const useAddSecretDraft = create<AddSecretDraft>((set) => ({
  ...initial,
  update: (patch) => set(patch),
  reset: () => set(initial),
}))
