import { useNavigate } from '@tanstack/react-router'
import { Check, ChevronDown, Database, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

import { ConfirmDialog } from '@/components/general/confirm-dialog'
import { GithubMark } from '@/components/general/github-mark'
import { RemoveVaultDialog } from '@/components/general/remove-vault-dialog'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useGhAccounts, useSwitchAccount } from '@/hooks/use-auth'
import { useConnectVault, useRemoteVaults, useVaults } from '@/hooks/use-vaults'
import { groupVaultsByOwner } from '@/lib/group-vaults-by-owner'
import { useAddSecretDraft } from '@/stores/add-secret-draft'
import type { VaultSummary } from '@/types'

/** Confirm-then-open flow; opening a vault also switches to the GitHub account that owns it. */
function useVaultSwitch(menuOpen: boolean) {
  const navigate = useNavigate()
  const [target, setTarget] = useState<VaultSummary | null>(null)
  const connect = useConnectVault()
  const switchAccount = useSwitchAccount()
  const { data: accounts = [] } = useGhAccounts(menuOpen)
  const resetDraft = useAddSecretDraft((s) => s.reset)

  const ownerLogin = target?.repo?.split('/')[0]
  // Vaults live under the GitHub account that owns the repo, so opening one means acting as that account.
  const accountToSwitchTo = accounts.find((a) => a.login === ownerLogin && !a.current)?.login

  const open = ({ id, repo }: VaultSummary) => {
    if (id) return void navigate({ to: '/vaults/$vaultId', params: { vaultId: id } })
    if (!repo) return
    connect.mutate(repo, {
      onSuccess: (vault) =>
        void navigate({ to: '/vaults/$vaultId', params: { vaultId: vault.id ?? '' } }),
      onError: (error) => toast.error(error.message),
    })
  }

  const confirm = () => {
    if (!target) return
    // Drop any half-typed secret so its value never lingers in memory after leaving the vault.
    resetDraft()
    setTarget(null)
    if (!accountToSwitchTo) return open(target)
    switchAccount.mutate(accountToSwitchTo, {
      onSuccess: () => open(target),
      onError: (error) => toast.error(error.message),
    })
  }

  return {
    target,
    accountToSwitchTo,
    busy: connect.isPending || switchAccount.isPending,
    select: setTarget,
    confirm,
  }
}

/** Connected vaults plus (once the menu opens) the ones that only exist on GitHub. */
function useVaultList(menuOpen: boolean, vaultId: string) {
  const { data } = useVaults()
  const remote = useRemoteVaults(menuOpen)
  return {
    vaults: [...(data?.vaults ?? []), ...(remote.data?.vaults ?? [])],
    current: data?.vaults.find((v) => v.id === vaultId),
  }
}

/** Dropdown in the header to open another vault, connect one from GitHub, or create a new one. */
export function VaultSwitcher({ vaultId }: { vaultId: string }) {
  const navigate = useNavigate()
  const [menuOpen, setMenuOpen] = useState(false)
  const [removing, setRemoving] = useState(false)
  const { vaults, current } = useVaultList(menuOpen, vaultId)
  const sw = useVaultSwitch(menuOpen)

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
          <VaultGroups vaults={vaults} activeId={vaultId} onSelect={sw.select} />
          <DropdownMenuSeparator />
          <DropdownMenuItem
            className="gap-3 py-2"
            onSelect={() => void navigate({ to: '/vaults/new/$step', params: { step: 'name' } })}
          >
            <Plus className="size-4 text-success" /> Create new vault
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
        open={sw.target !== null}
        title={`Switch to ${sw.target?.name ?? 'this vault'}?`}
        description={`You'll leave ${current?.name ?? 'the current vault'}${sw.accountToSwitchTo ? ` and switch the GitHub account to ${sw.accountToSwitchTo}` : ''}. Anything you haven't saved yet, such as a secret you're adding, will be discarded.`}
        confirmLabel="Switch vault"
        busy={sw.busy}
        onConfirm={sw.confirm}
        onCancel={() => sw.select(null)}
      />
    </>
  )
}

function VaultGroups(props: {
  vaults: VaultSummary[]
  activeId: string
  onSelect: (vault: VaultSummary) => void
}) {
  return groupVaultsByOwner(props.vaults).map((group, index) => (
    <DropdownMenuGroup key={group.owner ?? 'local'}>
      {index > 0 && <DropdownMenuSeparator />}
      <DropdownMenuLabel className="flex items-center gap-2 text-[11px] tracking-wide text-muted-foreground uppercase">
        {group.owner ? <GithubMark className="size-3" /> : null}
        {group.owner ?? 'Local only'}
        <span className="ml-auto font-normal">{group.vaults.length}</span>
      </DropdownMenuLabel>
      {group.vaults.map((vault) => (
        <VaultMenuItem
          key={vault.id ?? vault.repo}
          vault={vault}
          active={vault.id === props.activeId}
          // Re-selecting the open vault is a no-op, so it needs no warning.
          onSelect={() => (vault.id === props.activeId ? undefined : props.onSelect(vault))}
        />
      ))}
    </DropdownMenuGroup>
  ))
}

function VaultMenuItem({
  vault,
  active,
  onSelect,
}: {
  vault: VaultSummary
  active: boolean
  onSelect: () => void
}) {
  return (
    <DropdownMenuItem className="gap-3 py-2" onSelect={onSelect}>
      <span className="flex size-8 items-center justify-center rounded-lg bg-muted">
        <Database className="size-4" />
      </span>
      <span className="flex flex-1 flex-col">
        <span className="font-mono text-[13px] font-medium">{vault.name}</span>
        <span className="text-xs text-muted-foreground">
          {vault.repo?.split('/')[1] ?? 'Local only'}
          {vault.id ? '' : ' · click to connect'}
        </span>
      </span>
      {active && <Check className="size-4 text-success" aria-label="Current vault" />}
    </DropdownMenuItem>
  )
}
