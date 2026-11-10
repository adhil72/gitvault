import { useMutation, useQuery } from '@tanstack/react-query'
import { useEffect } from 'react'
import { toast } from 'sonner'

import { appVersion, checkForUpdate, installUpdate } from '@/api/updater'

/** Checks for a newer release on demand (`refetch`). Never runs on its own. */
export function useUpdateCheck() {
  return useQuery({
    queryKey: ['update-check'],
    queryFn: checkForUpdate,
    enabled: false,
    retry: false,
    staleTime: Infinity,
  })
}

export function useInstallUpdate() {
  return useMutation({ mutationFn: installUpdate })
}

export function useAppVersion() {
  return useQuery({ queryKey: ['app-version'], queryFn: appVersion, staleTime: Infinity })
}

/** Checks once at startup and offers the update in a toast. A failed check stays silent. */
export function useStartupUpdateNotice() {
  useEffect(() => {
    void checkForUpdate()
      .then((update) => {
        if (!update) return
        toast(`GitVault ${update.version} is available`, {
          duration: Infinity,
          action: { label: 'Install and restart', onClick: () => void installUpdate() },
        })
      })
      .catch(() => undefined)
  }, [])
}
