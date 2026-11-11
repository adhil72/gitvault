import type { QueryClient } from '@tanstack/react-query'

import { vaultInfo, vaultUnlockTrusted } from '@/api/vaults'
import { queryKeys } from '@/config/query-keys'

async function unlockedByTrust(vaultId: string): Promise<boolean> {
  try {
    await vaultUnlockTrusted(vaultId)
    return true
  } catch {
    return false
  }
}

/**
 * Opens a vault without asking for a password. A vault is locked by default: pages show what is
 * readable without the key and ask for the password in a dialog when it is needed. A device the
 * user chose to trust unlocks silently here.
 */
export async function vaultGuard(queryClient: QueryClient, vaultId: string): Promise<void> {
  const vault = await queryClient.fetchQuery({
    queryKey: queryKeys.vault(vaultId),
    queryFn: () => vaultInfo(vaultId),
    staleTime: 0,
  })
  if (!vault.unlocked && (await unlockedByTrust(vaultId))) {
    void queryClient.invalidateQueries({ queryKey: queryKeys.vault(vaultId) })
  }
}
