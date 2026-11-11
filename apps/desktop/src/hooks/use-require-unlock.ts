import { useVault } from '@/hooks/use-vaults'
import { useUnlockRequest } from '@/stores/unlock-request'

/**
 * Wraps an action that needs the vault key (reveal, copy, edit, ...).
 * Runs it at once when the vault is unlocked, otherwise asks for the password first.
 */
export function useRequireUnlock(vaultId: string) {
  const { data: vault } = useVault(vaultId)
  const request = useUnlockRequest((s) => s.request)
  return (action?: () => void) => {
    if (vault?.unlocked) action?.()
    else request(vaultId, action)
  }
}
