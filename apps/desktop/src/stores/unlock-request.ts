import { create } from 'zustand'

/** A request to unlock a vault, with what to do once it is unlocked. In memory only; holds no secrets. */
interface UnlockRequestStore {
  vaultId: string | null
  then: (() => void) | null
  request: (vaultId: string, then?: () => void) => void
  close: () => void
}

export const useUnlockRequest = create<UnlockRequestStore>((set) => ({
  vaultId: null,
  then: null,
  request: (vaultId, then) => set({ vaultId, then: then ?? null }),
  close: () => set({ vaultId: null, then: null }),
}))
