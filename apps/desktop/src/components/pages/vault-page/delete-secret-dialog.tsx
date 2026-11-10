import { useState } from 'react'

import { ConfirmDialog } from '@/components/general/confirm-dialog'
import { ErrorNotice } from '@/components/general/error-notice'
import { Input } from '@/components/ui/input'
import { useDeleteSecret, useSecret } from '@/hooks/use-secrets'

import type { useVaultNav } from './use-vault-nav'

/** Asks the user to type the secret's name before deleting it. */
export function DeleteSecretDialog({ nav }: { nav: ReturnType<typeof useVaultNav> }) {
  const open = nav.search.panel === 'delete' && Boolean(nav.search.secret)
  const { data: secret } = useSecret(nav.vaultId, open ? nav.search.secret : undefined)
  const remove = useDeleteSecret(nav.vaultId)
  const [typed, setTyped] = useState('')

  const close = () => {
    setTyped('')
    remove.reset()
    nav.closePanel()
  }

  return (
    <ConfirmDialog
      open={open}
      title="Delete this secret?"
      description="It is removed from the vault and a commit is pushed to GitHub. You can restore it from version history."
      confirmLabel="Delete secret"
      destructive
      busy={remove.isPending}
      confirmDisabled={!secret || typed !== secret.name}
      onCancel={close}
      onConfirm={() =>
        secret &&
        remove.mutate(secret.id, {
          onSuccess: () => {
            setTyped('')
            nav.clearSelection()
          },
        })
      }
    >
      <p className="font-mono text-sm text-muted-foreground">{secret?.name}</p>
      <label htmlFor="confirm-name" className="text-sm font-semibold">
        Type the secret name to confirm
      </label>
      <Input
        id="confirm-name"
        value={typed}
        onChange={(e) => setTyped(e.target.value)}
        className="h-10 bg-inset font-mono"
        autoComplete="off"
      />
      <ErrorNotice error={remove.error} />
    </ConfirmDialog>
  )
}
