import { useNavigate } from '@tanstack/react-router'
import { Database } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { useConnectVault } from '@/hooks/use-vaults'
import type { VaultSummary } from '@/types'

/** One vault in the list: open it (it opens locked) or connect it from GitHub. */
export function VaultCard({ vault }: { vault: VaultSummary }) {
  const navigate = useNavigate()
  const connect = useConnectVault()

  const open = () => {
    if (!vault.id) return
    void navigate({ to: '/vaults/$vaultId', params: { vaultId: vault.id } })
  }

  const connectRemote = () => {
    if (!vault.repo) return
    connect.mutate(vault.repo, {
      onSuccess: (connected) =>
        void navigate({ to: '/vaults/$vaultId', params: { vaultId: connected.id ?? '' } }),
      onError: (error) => toast.error(error.message),
    })
  }

  return (
    <li className="flex items-center gap-3.5 rounded-lg border border-border bg-card p-4">
      <span className="flex size-9 items-center justify-center rounded-lg bg-muted">
        <Database className="size-[18px]" />
      </span>
      <span className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="truncate font-mono text-sm font-medium">{vault.name}</span>
        <span className="truncate text-xs text-muted-foreground">
          {vault.repo ?? 'Local only'} ·{' '}
          {vault.id ? (vault.unlocked ? 'unlocked' : 'locked') : 'on GitHub, not connected'}
        </span>
      </span>
      {vault.id ? (
        <Button variant="secondary" size="sm" onClick={open}>
          Open
        </Button>
      ) : (
        <Button size="sm" onClick={connectRemote} disabled={connect.isPending}>
          {connect.isPending ? 'Connecting…' : 'Connect'}
        </Button>
      )}
    </li>
  )
}
