import { create } from 'zustand'

/** In-memory draft of the create-vault wizard. The master password is never stored here. */
interface CreatedResult {
  vaultId: string
  vaultName: string
  recoveryKey: string
  repo: string | null
  remoteError: string | null
}

interface CreateVaultDraft {
  name: string
  description: string
  createRemote: boolean
  repoName: string
  result: CreatedResult | null
  update: (patch: Partial<Omit<CreateVaultDraft, 'update' | 'reset'>>) => void
  reset: () => void
}

const initial = { name: '', description: '', createRemote: true, repoName: '', result: null }

export const useCreateVaultDraft = create<CreateVaultDraft>((set) => ({
  ...initial,
  update: (patch) => set(patch),
  reset: () => set(initial),
}))
