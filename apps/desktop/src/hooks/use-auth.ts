import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { authLoginGh, authLogout, authStatus } from '@/api/auth'
import { queryKeys } from '@/config/query-keys'

export const authQueryOptions = () =>
  queryOptions({ queryKey: queryKeys.auth, queryFn: authStatus, staleTime: Infinity })

export const useAccount = () => useQuery(authQueryOptions())

export function useLoginWithGh() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: authLoginGh,
    onSuccess: (account) => {
      queryClient.setQueryData(queryKeys.auth, account)
      void queryClient.invalidateQueries({ queryKey: queryKeys.vaults })
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
