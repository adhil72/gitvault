import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { syncConflicts, syncNow, syncResolve, syncStatus } from '@/api/sync'
import { queryKeys } from '@/config/query-keys'
import type { ConflictKeep } from '@/types'

export const useSyncStatus = (vaultId: string) =>
  useQuery({
    queryKey: queryKeys.sync(vaultId),
    queryFn: () => syncStatus(vaultId),
    refetchInterval: 30_000,
  })

export const useConflicts = (vaultId: string) =>
  useQuery({ queryKey: queryKeys.conflicts(vaultId), queryFn: () => syncConflicts(vaultId) })

export function useSyncNow(vaultId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => syncNow(vaultId),
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.sync(vaultId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.secrets(vaultId) })
    },
  })
}

export function useResolveConflict(vaultId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (v: { secretId: string; keep: ConflictKeep }) =>
      syncResolve(vaultId, v.secretId, v.keep),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.sync(vaultId) })
      void queryClient.invalidateQueries({ queryKey: queryKeys.secrets(vaultId) })
    },
  })
}
