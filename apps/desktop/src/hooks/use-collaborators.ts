import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { collaboratorInvite, collaboratorList, collaboratorRemove } from '@/api/vaults'
import type { CollaboratorRole } from '@/types'

const collaboratorsKey = (vaultId: string) => ['vault', vaultId, 'collaborators'] as const

export const useCollaborators = (vaultId: string) =>
  useQuery({ queryKey: collaboratorsKey(vaultId), queryFn: () => collaboratorList(vaultId) })

export function useInviteCollaborator(vaultId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (invite: { login: string; role: CollaboratorRole }) =>
      collaboratorInvite({ vault_id: vaultId, ...invite }),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: collaboratorsKey(vaultId) }),
  })
}

export function useRemoveCollaborator(vaultId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (login: string) => collaboratorRemove(vaultId, login),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: collaboratorsKey(vaultId) }),
  })
}
