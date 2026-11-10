import { useNavigate } from '@tanstack/react-router'
import { ChevronDown, Database, Plus } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

import { GithubMark } from '@/components/general/github-mark'
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
import type { VaultSummary } from '@/types'

/** Dropdown in the header to open another vault, connect one from GitHub, or create a new one. */
export function VaultSwitcher({ vaultId }: { vaultId: string }) {
  const navigate = useNavigate()
  const [menuOpen, setMenuOpen] = useState(false)
  const { data } = useVaults()
  const remote = useRemoteVaults(menuOpen)
  const vaults = [...(data?.vaults ?? []), ...(remote.data?.vaults ?? [])]
  const connect = useConnectVault()
  const current = data?.vaults.find((v) => v.id === vaultId)

  const open = (vault: VaultSummary) => {
    if (vault.id) void navigate({ to: '/vaults/$vaultId', params: { vaultId: vault.id } })
  }

  const connectRemote = (repo: string | null) => {
    if (!repo) return
    connect.mutate(repo, {
      onSuccess: (vault) =>
        void navigate({
          to: '/vaults/$vaultId/unlock',
          params: { vaultId: vault.id ?? '' },
          search: {},
        }),
      onError: (error) => toast.error(error.message),
    })
  }

  return (
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
          <DropdownMenuItem
            key={vault.id ?? vault.repo}
            className="gap-3 py-2"
            onSelect={() => (vault.id ? open(vault) : connectRemote(vault.repo))}
          >
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
          onSelect={() => void navigate({ to: '/vaults', search: { all: true } })}
        >
          <GithubMark className="size-4" /> All vaults…
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
