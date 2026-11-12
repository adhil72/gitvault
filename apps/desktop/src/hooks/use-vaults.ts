import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import {
  vaultChangePassword,
  vaultConnect,
  vaultCreate,
  vaultDiscover,
  vaultForget,
  vaultInfo,
  vaultList,
  vaultLock,
  vaultRemoteStatus,
  vaultRepublish,
  vaultRegenerateRecovery,
  vaultUnlock,
  vaultUse,
} from '@/api/vaults'
import { queryKeys } from '@/config/query-keys'

export const vaultsQueryOptions = () =>
  queryOptions({ queryKey: queryKeys.vaults, queryFn: vaultList })

export const vaultQueryOptions = (vaultId: string) =>
  queryOptions({ queryKey: queryKeys.vault(vaultId), queryFn: () => vaultInfo(vaultId) })

export const useVaults = () => useQuery(vaultsQueryOptions())

/** Vaults on GitHub that are not cloned yet. Only fetched on request: opening the app stays offline-fast. */
export const useRemoteVaults = (enabled: boolean) =>
  useQuery({
    queryKey: queryKeys.remoteVaults,
    queryFn: vaultDiscover,
    enabled,
    staleTime: 5 * 60_000,
  })

/** Polls the vault so an auto-lock (idle timeout) is noticed by the UI. */
export const useVault = (vaultId: string) =>
  useQuery({ ...vaultQueryOptions(vaultId), refetchInterval: 20_000 })

/** Asks GitHub whether the vault's repository still exists. Needs the network, so it never retries. */
export const useRemoteStatus = (vaultId: string) =>
  useQuery({
    queryKey: queryKeys.remoteStatus(vaultId),
    queryFn: () => vaultRemoteStatus(vaultId),
    staleTime: 60_000,
    retry: false,
  })

function useVaultMutation<TVariables, TResult>(
  mutationFn: (variables: TVariables) => Promise<TResult>,
) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.vaults })
      void queryClient.invalidateQueries({ queryKey: ['vault'] })
      // Locking or unlocking changes what secret reads return (the local name index has no notes).
      void queryClient.invalidateQueries({ queryKey: ['secrets'] })
    },
  })
}

export const useCreateVault = () => useVaultMutation(vaultCreate)
export const useConnectVault = () => useVaultMutation(vaultConnect)
export const useUnlockVault = () => useVaultMutation(vaultUnlock)
export const useLockVault = () => useVaultMutation(vaultLock)
export const useUseVault = () => useVaultMutation(vaultUse)
export const useRepublishVault = () => useVaultMutation(vaultRepublish)
export const useForgetVault = () => useVaultMutation(vaultForget)

export const useChangePassword = () =>
  useMutation({
    mutationFn: (v: { vaultId: string; current: string; newPassword: string }) =>
      vaultChangePassword(v.vaultId, v.current, v.newPassword),
  })

export const useRegenerateRecovery = () =>
  useMutation({
    mutationFn: (v: { vaultId: string; current: string }) =>
      vaultRegenerateRecovery(v.vaultId, v.current),
  })
