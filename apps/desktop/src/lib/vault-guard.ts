import type { QueryClient } from '@tanstack/react-query'
import { redirect } from '@tanstack/react-router'

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

/** Redirects to the unlock page when the vault is locked (always checks the backend). */
export async function vaultGuard(queryClient: QueryClient, vaultId: string): Promise<void> {
  const vault = await queryClient.fetchQuery({
    queryKey: queryKeys.vault(vaultId),
    queryFn: () => vaultInfo(vaultId),
    staleTime: 0,
  })
  // A trusted device unlocks silently; anything else falls through to the password page.
  if (!vault.unlocked && (await unlockedByTrust(vaultId))) {
    void queryClient.invalidateQueries({ queryKey: queryKeys.vault(vaultId) })
    return
  }
  if (!vault.unlocked) {
    throw redirect({ to: '/vaults/$vaultId/unlock', params: { vaultId }, search: {} })
  }
}
