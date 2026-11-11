import { Lock } from 'lucide-react'

import { EmptyState } from '@/components/general/empty-state'
import { Button } from '@/components/ui/button'
import { useRequireUnlock } from '@/hooks/use-require-unlock'
import { useVault } from '@/hooks/use-vaults'

/** Shown above the list while the vault is locked: names are visible, values are not. */
function LockedBanner({ onUnlock }: { onUnlock: () => void }) {
  return (
    <div className="mx-6 mb-4 flex items-center gap-3 rounded-lg border border-border bg-muted/40 px-4 py-3">
      <Lock className="size-4 shrink-0 text-muted-foreground" />
      <p className="flex-1 text-[13px] text-muted-foreground">
        This vault is locked. Enter the master password to view, copy or change secrets.
      </p>
      <Button size="sm" onClick={onUnlock}>
        Unlock
      </Button>
    </div>
  )
}

/** Shown when a locked vault has nothing readable yet (this device never unlocked it). */
function LockedPlaceholder({ onUnlock }: { onUnlock: () => void }) {
  return (
    <EmptyState
      icon={Lock}
      title="This vault is locked"
      description="Secret names are encrypted. Unlock the vault once on this device to see them."
    >
      <Button onClick={onUnlock}>Unlock vault</Button>
    </EmptyState>
  )
}

/** Lock state of the list: a banner over visible names, or a placeholder when nothing is readable. */
export function LockedNotice({
  vaultId,
  nothingReadable,
}: {
  vaultId: string
  nothingReadable: boolean
}) {
  const { data: vault } = useVault(vaultId)
  const requireUnlock = useRequireUnlock(vaultId)
  if (nothingReadable) return <LockedPlaceholder onUnlock={() => requireUnlock()} />
  if (vault && !vault.unlocked) return <LockedBanner onUnlock={() => requireUnlock()} />
  return null
}
