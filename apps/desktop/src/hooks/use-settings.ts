import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTheme } from 'next-themes'

import { settingsGet, settingsSet } from '@/api/tools'
import { queryKeys } from '@/config/query-keys'
import type { Settings } from '@/types'

export const settingsQueryOptions = () =>
  queryOptions({ queryKey: queryKeys.settings, queryFn: settingsGet, staleTime: Infinity })

export const useSettings = () => useQuery(settingsQueryOptions())

/** Saves settings and applies the theme immediately. */
export function useUpdateSettings() {
  const queryClient = useQueryClient()
  const { setTheme } = useTheme()
  return useMutation({
    mutationFn: (settings: Settings) => settingsSet(settings),
    onSuccess: (saved) => {
      queryClient.setQueryData(queryKeys.settings, saved)
      setTheme(saved.theme)
    },
  })
}
