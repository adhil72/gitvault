import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import { secretMarkCopied, secretReveal } from '@/api/secrets'
import { clearClipboard, writeClipboard } from '@/api/system'
import { queryKeys } from '@/config/query-keys'
import type { Settings } from '@/types'

let clearTimer: ReturnType<typeof setTimeout> | undefined

/** Copies a secret to the clipboard, records it in the audit log and clears it after a delay. */
export function useCopySecret(vaultId: string) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (secretId: string) => {
      const { value } = await secretReveal(vaultId, secretId)
      await writeClipboard(value)
      await secretMarkCopied(vaultId, secretId)
    },
    onSuccess: () => {
      const seconds =
        queryClient.getQueryData<Settings>(queryKeys.settings)?.clear_clipboard_seconds ?? 0
      clearTimeout(clearTimer)
      if (seconds > 0) clearTimer = setTimeout(() => void clearClipboard(), seconds * 1000)
      toast.success(seconds > 0 ? `Copied. Clipboard clears in ${seconds}s.` : 'Copied.')
    },
    onError: (error: Error) => toast.error(error.message),
  })
}
