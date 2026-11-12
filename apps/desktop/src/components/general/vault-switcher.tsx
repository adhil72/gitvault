import { useNavigate } from '@tanstack/react-router'
import { ChevronDown, Database, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

import { ConfirmDialog } from '@/components/general/confirm-dialog'
import { GithubMark } from '@/components/general/github-mark'
import { RemoveVaultDialog } from '@/components/general/remove-vault-dialog'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useConnectVault, useRemoteVaults, useVaults } from '@/hooks/use-vaults'
import { useAddSecretDraft } from '@/stores/add-secret-draft'
import type { VaultSummary } from '@/types'

/** Dropdown in the header to open another vault, connect one from GitHub, or create a new one. */
export function VaultSwitcher({ vaultId }: { vaultId: string }) {
  const navigate = useNavigate()
  const [menuOpen, setMenuOpen] = useState(false)
  const [target, setTarget] = useState<VaultSummary | null>(null)
  const [removing, setRemoving] = useState(false)
  const { data } = useVaults()
  const remote = useRemoteVaults(menuOpen)
  const vaults = [...(data?.vaults ?? []), ...(remote.data?.vaults ?? [])]
  const connect = useConnectVault()
  const resetDraft = useAddSecretDraft((s) => s.reset)
  const current = data?.vaults.find((v) => v.id === vaultId)

  const connectRemote = (repo: string | null) => {
    if (!repo) return
    connect.mutate(repo, {
      onSuccess: (vault) =>
        void navigate({ to: '/vaults/$vaultId', params: { vaultId: vault.id ?? '' } }),
      onError: (error) => toast.error(error.message),
    })
  }

  const confirmSwitch = () => {
    if (!target) return
    // Drop any half-typed secret so its value never lingers in memory after leaving the vault.
    resetDraft()
    setTarget(null)
    if (target.id) void navigate({ to: '/vaults/$vaultId', params: { vaultId: target.id } })
    else connectRemote(target.repo)
  }

  return (
    <>
      <DropdownMenu onOpenChange={setMenuOpen}>
        <DropdownMenuTrigger asChild>
          <Button variant="secondary" className="h-9 rounded-full px-4" aria-label="Switch vault">
            {current?.name ?? 'Vault'}
            <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-80">
          <DropdownMenuLabel className="text-[11px] tracking-wide text-muted-foreground uppercase">
            Your vaults
          </DropdownMenuLabel>
          {vaults.map((vault) => (
            <VaultMenuItem
              key={vault.id ?? vault.repo}
              vault={vault}
              // Re-selecting the open vault is a no-op, so it needs no warning.
              onSelect={() => (vault.id === vaultId ? undefined : setTarget(vault))}
            />
          ))}
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3 py-2"
            onSelect={() => void navigate({ to: '/vaults/new/$step', params: { step: 'name' } })}
          >
            <Plus className="size-4 text-success" /> Create new vault
          </DropdownMenuItem>
          <DropdownMenuItem
            className="gap-3 py-2"
            onSelect={() => void navigate({ to: '/vaults' })}
          >
            <GithubMark className="size-4" /> All vaults…
          </DropdownMenuItem>
          <DropdownMenuItem
            variant="destructive"
            className="gap-3 py-2"
            onSelect={() => setRemoving(true)}
          >
            <Trash2 className="size-4" /> Remove from this device
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <RemoveVaultDialog open={removing} vaultId={vaultId} onClose={() => setRemoving(false)} />
      <ConfirmDialog
        open={target !== null}
        title={`Switch to ${target?.name ?? 'this vault'}?`}
        description={`You'll leave ${current?.name ?? 'the current vault'}. Anything you haven't saved yet, such as a secret you're adding, will be discarded.`}
        confirmLabel="Switch vault"
        busy={connect.isPending}
        onConfirm={confirmSwitch}
        onCancel={() => setTarget(null)}
      />
    </>
  )
}

function VaultMenuItem({ vault, onSelect }: { vault: VaultSummary; onSelect: () => void }) {
  return (
    <DropdownMenuItem className="gap-3 py-2" onSelect={onSelect}>
      <span className="flex size-8 items-center justify-center rounded-lg bg-muted">
        <Database className="size-4" />
      </span>
      <span className="flex flex-1 flex-col">
        <span className="font-mono text-[13px] font-medium">{vault.name}</span>
        <span className="text-xs text-muted-foreground">
          {vault.repo ?? 'Local only'}
          {vault.id ? '' : ' · click to connect'}
        </span>
      </span>
    </DropdownMenuItem>
  )
}
