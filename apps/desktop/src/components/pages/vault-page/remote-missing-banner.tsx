import { TriangleAlert } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

import { Callout } from '@/components/general/callout'
import { RemoveVaultDialog } from '@/components/general/remove-vault-dialog'
import { Button } from '@/components/ui/button'
import { useRemoteStatus, useRepublishVault } from '@/hooks/use-vaults'

/** Shown when the vault's GitHub repository was deleted: offers to recreate it or drop the local copy. */
export function RemoteMissingBanner({ vaultId }: { vaultId: string }) {
  const { data: status } = useRemoteStatus(vaultId)
  const republish = useRepublishVault()
  const [removing, setRemoving] = useState(false)
  if (status !== 'missing') return null

  const recreate = () =>
    republish.mutate(vaultId, {
      onSuccess: () => toast.success('Repository recreated and vault pushed'),
      onError: (error) => toast.error(error.message),
    })

  return (
    <div className="mx-6 mb-4">
      <Callout tone="warning" icon={<TriangleAlert className="size-4" />}>
        <p className="font-medium">The GitHub repository was deleted</p>
        <p className="mt-1 text-muted-foreground">
          This device holds the only copy of the vault. Recreate the repository to back it up again,
          or remove the vault from this device.
        </p>
        <div className="mt-3 flex gap-2">
          <Button size="sm" disabled={republish.isPending} onClick={recreate}>
            {republish.isPending ? 'Recreating…' : 'Recreate repository'}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => setRemoving(true)}>
            Remove from this device
          </Button>
        </div>
      </Callout>
      <RemoveVaultDialog open={removing} vaultId={vaultId} onClose={() => setRemoving(false)} />
    </div>
  )
}
