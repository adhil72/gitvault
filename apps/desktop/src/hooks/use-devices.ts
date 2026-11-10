import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { deviceList, deviceRevoke, deviceTrust } from '@/api/vaults'

const devicesKey = (vaultId: string) => ['vault', vaultId, 'devices'] as const

export const useDevices = (vaultId: string) =>
  useQuery({ queryKey: devicesKey(vaultId), queryFn: () => deviceList(vaultId) })

export function useTrustDevice(vaultId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => deviceTrust(vaultId, name),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: devicesKey(vaultId) }),
  })
}

export function useRevokeDevice(vaultId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (deviceId: string) => deviceRevoke(vaultId, deviceId),
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: devicesKey(vaultId) }),
  })
}
