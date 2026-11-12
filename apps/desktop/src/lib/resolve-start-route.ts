import type { QueryClient } from '@tanstack/react-query'

import { vaultList } from '@/api/vaults'
import { queryKeys } from '@/config/query-keys'
import { authQueryOptions } from '@/hooks/use-auth'
import { pickVaultForAccount } from '@/lib/pick-vault-for-account'

export type StartRoute =
  { to: '/vaults/$vaultId'; params: { vaultId: string } } | { to: '/no-vault' }

/** Where the app opens: the signed-in account's last used (or first) vault, else the empty page. */
export async function resolveStartRoute(queryClient: QueryClient): Promise<StartRoute> {
  const account = await queryClient.ensureQueryData(authQueryOptions())
  const list = await queryClient
    .fetchQuery({ queryKey: queryKeys.vaults, queryFn: vaultList, staleTime: 0 })
    .catch(() => null)
  const vault = account && list ? pickVaultForAccount(list.vaults, account.login) : null
  return vault?.id ? { to: '/vaults/$vaultId', params: { vaultId: vault.id } } : { to: '/no-vault' }
}
