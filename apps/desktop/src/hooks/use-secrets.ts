import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import {
  importEnv,
  importPreview,
  secretAdd,
  secretDelete,
  secretFacets,
  secretGet,
  secretHistory,
  secretList,
  secretReveal,
  secretRestore,
  secretRotate,
  secretUpdate,
} from '@/api/secrets'
import { syncNow } from '@/api/sync'
import { queryKeys } from '@/config/query-keys'
import type { ImportEnvInput, ListSecretsQuery, Settings } from '@/types'

export const secretsQueryOptions = (query: ListSecretsQuery) =>
  queryOptions({ queryKey: queryKeys.secretList(query), queryFn: () => secretList(query) })

export const facetsQueryOptions = (vaultId: string) =>
  queryOptions({ queryKey: queryKeys.facets(vaultId), queryFn: () => secretFacets(vaultId) })

export const useSecrets = (query: ListSecretsQuery) => useQuery(secretsQueryOptions(query))
export const useFacets = (vaultId: string) => useQuery(facetsQueryOptions(vaultId))

export const useSecret = (vaultId: string, secretId: string | undefined) =>
  useQuery({
    queryKey: queryKeys.secret(vaultId, secretId ?? ''),
    queryFn: () => secretGet(vaultId, secretId ?? ''),
    enabled: Boolean(secretId),
  })

export const useHistory = (vaultId: string, secretId: string) =>
  useQuery({
    queryKey: queryKeys.history(vaultId, secretId),
    queryFn: () => secretHistory(vaultId, secretId),
  })

/** Refreshes secret data and, when auto-sync is on, pushes the change to GitHub. */
function useAfterWrite(vaultId: string) {
  const queryClient = useQueryClient()
  return async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.secrets(vaultId) })
    const settings = queryClient.getQueryData<Settings>(queryKeys.settings)
    const refresh = () => queryClient.invalidateQueries({ queryKey: queryKeys.sync(vaultId) })
    if (settings?.auto_sync) void syncNow(vaultId).then(refresh, refresh)
    else void refresh()
  }
}

export function useAddSecret(vaultId: string) {
  const afterWrite = useAfterWrite(vaultId)
  return useMutation({ mutationFn: secretAdd, onSuccess: afterWrite })
}

export function useUpdateSecret(vaultId: string) {
  const afterWrite = useAfterWrite(vaultId)
  return useMutation({ mutationFn: secretUpdate, onSuccess: afterWrite })
}

export function useRotateSecret(vaultId: string) {
  const afterWrite = useAfterWrite(vaultId)
  return useMutation({
    mutationFn: (v: { secretId: string; value: string }) =>
      secretRotate(vaultId, v.secretId, v.value),
    onSuccess: afterWrite,
  })
}

export function useDeleteSecret(vaultId: string) {
  const afterWrite = useAfterWrite(vaultId)
  return useMutation({
    mutationFn: (secretId: string) => secretDelete(vaultId, secretId),
    onSuccess: afterWrite,
  })
}

export function useRestoreSecret(vaultId: string) {
  const afterWrite = useAfterWrite(vaultId)
  return useMutation({
    mutationFn: (v: { secretId: string; commit: string }) =>
      secretRestore(vaultId, v.secretId, v.commit),
    onSuccess: afterWrite,
  })
}

/** Fetches a plaintext value on demand. The result is never cached. */
export const useRevealSecret = (vaultId: string) =>
  useMutation({ mutationFn: (secretId: string) => secretReveal(vaultId, secretId), gcTime: 0 })

export const useImportPreview = () =>
  useMutation({ mutationFn: (input: ImportEnvInput) => importPreview(input) })

export function useImportEnv(vaultId: string) {
  const afterWrite = useAfterWrite(vaultId)
  return useMutation({ mutationFn: importEnv, onSuccess: afterWrite })
}
