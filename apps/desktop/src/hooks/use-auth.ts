import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { authGhAccounts, authLoginGh, authLoginGhAccount, authLogout, authStatus } from '@/api/auth'
import { queryKeys } from '@/config/query-keys'
import type { AccountDto } from '@/types'

/** The stored session, or (on first read) a silent sign-in with the GitHub CLI account when one exists. */
async function currentAccount(): Promise<AccountDto | null> {
  const stored = await authStatus()
  if (stored) return stored
  const accounts = await authGhAccounts().catch(() => [])
  if (accounts.length === 0) return null
  return authLoginGh().catch(() => null)
}

export const authQueryOptions = () =>
  queryOptions({ queryKey: queryKeys.auth, queryFn: currentAccount, staleTime: Infinity })

export const useAccount = () => useQuery(authQueryOptions())

/** Accounts known to the local `gh` CLI. Only fetched on request. */
export const useGhAccounts = (enabled = true) =>
  useQuery({ queryKey: queryKeys.ghAccounts, queryFn: authGhAccounts, enabled, staleTime: 0 })

export function useLoginWithGh() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (login?: string) => (login ? authLoginGhAccount(login) : authLoginGh()),
    onSuccess: (account) => {
      queryClient.setQueryData(queryKeys.auth, account)
      void queryClient.invalidateQueries({ queryKey: queryKeys.vaults })
    },
  })
}

/** Switches to another `gh` account: everything fetched for the old account is dropped. */
export function useSwitchAccount() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: authLoginGhAccount,
    onSuccess: (account) => {
      queryClient.removeQueries({ predicate: (q) => q.queryKey[0] !== 'auth' })
      queryClient.setQueryData(queryKeys.auth, account)
      void queryClient.invalidateQueries({ queryKey: queryKeys.ghAccounts })
    },
  })
}

export function useLogout() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: authLogout,
    onSuccess: () => {
      queryClient.setQueryData(queryKeys.auth, null)
      void queryClient.invalidateQueries({ queryKey: queryKeys.vaults })
    },
  })
}
