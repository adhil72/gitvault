import { Link } from '@tanstack/react-router'
import { RefreshCw } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { useSyncNow, useSyncStatus } from '@/hooks/use-sync'
import { timeAgo } from '@/lib/format'

/** Shows the sync state of a vault and lets the user sync now. */
export function SyncIndicator({ vaultId }: { vaultId: string }) {
  const { data } = useSyncStatus(vaultId)
  const sync = useSyncNow(vaultId)
  if (!data || data.state === 'no_remote') {
    return <span className="text-xs text-muted-foreground">Local only</span>
  }
  if (data.state === 'conflict') {
    return (
      <Button asChild variant="danger" size="sm">
        <Link to="/vaults/$vaultId/settings/$tab" params={{ vaultId, tab: 'github-sync' }}>
          {data.conflicts} conflict{data.conflicts === 1 ? '' : 's'}
        </Link>
      </Button>
    )
  }
  const label =
    data.state === 'pending'
      ? `${data.pending} to push`
      : data.last_sync
        ? `Synced ${timeAgo(data.last_sync)}`
        : 'Synced'
  return (
    <Button
      variant="ghost"
      size="sm"
      aria-label="Sync now"
      disabled={sync.isPending}
      onClick={() =>
        sync.mutate(undefined, {
          onSuccess: (r) =>
            toast.success(
              r.conflicts
                ? 'Synced with conflicts'
                : `Synced: ${r.pulled} pulled, ${r.pushed} pushed`,
            ),
          onError: (error) => toast.error(error.message),
        })
      }
    >
      <RefreshCw className={sync.isPending ? 'animate-spin' : ''} />
      {sync.isPending ? 'Syncing…' : label}
    </Button>
  )
}
