import { useNavigate } from '@tanstack/react-router'
import { Check, ChevronDown, Lock, LogOut, Settings } from 'lucide-react'
import { useState, type ComponentProps } from 'react'
import { toast } from 'sonner'

import { ConfirmDialog } from '@/components/general/confirm-dialog'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Skeleton } from '@/components/ui/skeleton'
import {
  useAccount,
  useGhAccounts,
  useLogout,
  usePrefetchGhAccounts,
  useSwitchAccount,
} from '@/hooks/use-auth'
import { useLockVault } from '@/hooks/use-vaults'
import { cn } from '@/lib/utils'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

function Avatar({
  login,
  active,
  className,
}: {
  login: string
  active?: boolean
  className?: string
}) {
  return (
    <span
      aria-hidden
      className={cn(
        'flex size-6 shrink-0 items-center justify-center rounded-full border text-xs font-medium',
        active
          ? 'border-done bg-done/10 text-done'
          : 'border-border bg-muted text-muted-foreground',
        className,
      )}
    >
      {login.slice(0, 1).toUpperCase()}
    </span>
  )
}

type TriggerProps = ComponentProps<typeof Button> & {
  login: string | undefined
  showName: boolean
}

// Spreads the rest so the dropdown's `asChild` trigger can attach its ref and handlers.
function AccountTrigger({ login, showName, ...rest }: TriggerProps) {
  const name = login ?? '?'
  if (showName) {
    return (
      <Button
        {...rest}
        variant="secondary"
        aria-label="Account menu"
        className="group h-9 rounded-full pr-3"
      >
        <Avatar login={name} active />
        {login}
        <ChevronDown className="transition-transform group-data-[state=open]:rotate-180" />
      </Button>
    )
  }
  return (
    <Button
      {...rest}
      variant="ghost"
      size="icon"
      aria-label="Account menu"
      className="rounded-full"
    >
      <Avatar login={name} active className="size-8 text-sm" />
    </Button>
  )
}

interface AccountItemsProps {
  accounts: { login: string }[]
  loading: boolean
  disabled: boolean
  onPick: (login: string) => void
}

/** The other `gh` accounts to switch to; the current one is shown by `CurrentAccount`. */
function AccountItems({ accounts, loading, disabled, onPick }: AccountItemsProps) {
  if (loading) return <Skeleton className="mx-2 my-1 h-8" />
  return accounts.map((gh) => (
    <DropdownMenuItem key={gh.login} disabled={disabled} onSelect={() => onPick(gh.login)}>
      <Avatar login={gh.login} />
      <span className="flex-1 truncate">{gh.login}</span>
    </DropdownMenuItem>
  ))
}

/** Always-visible header: who is signed in, available instantly from the auth cache. */
function CurrentAccount({ login }: { login: string | undefined }) {
  return (
    <div className="flex items-center gap-3 px-2 py-2">
      <Avatar login={login ?? '?'} active className="size-8 text-sm" />
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium">{login ?? 'Signed out'}</p>
        <p className="text-xs text-muted-foreground">GitHub account</p>
      </div>
      <Check className="size-4 text-done" aria-label="Current account" />
    </div>
  )
}

/** Settings and lock entries, only meaningful while a vault is open. */
function VaultActions({ vaultId }: { vaultId: string }) {
  const navigate = useNavigate()
  const lock = useLockVault()
  return (
    <>
      <DropdownMenuItem
        onSelect={() =>
          void navigate({
            to: '/vaults/$vaultId/settings/$tab',
            params: { vaultId, tab: 'general' },
          })
        }
      >
        <Settings /> Settings
      </DropdownMenuItem>
      <DropdownMenuItem onSelect={() => lock.mutate(vaultId)}>
        <Lock /> Lock vault
      </DropdownMenuItem>
      <DropdownMenuSeparator />
    </>
  )
}

/** Header menu: switch GitHub CLI account, settings, lock, sign out. */
export function AccountMenu({
  vaultId,
  showName = false,
}: {
  vaultId?: string | undefined
  /** Shows the login next to the avatar, so the switch-account control is obvious. */
  showName?: boolean
}) {
  const navigate = useNavigate()
  const [open, setOpen] = useState(false)
  const [pendingLogin, setPendingLogin] = useState<string | null>(null)
  const { data: account } = useAccount()
  const { data: accounts, isPending } = useGhAccounts(open)
  const prefetchAccounts = usePrefetchGhAccounts()
  const switchAccount = useSwitchAccount()
  const resetDraft = useAddSecretDraft((s) => s.reset)
  const logout = useLogout()

  const confirmSwitch = () => {
    if (!pendingLogin) return
    resetDraft()
    switchAccount.mutate(pendingLogin, {
      // /vaults resolves to the new account's last used vault, or the empty page.
      onSuccess: () => void navigate({ to: '/vaults' }),
      onError: (error) => toast.error(error.message),
      onSettled: () => setPendingLogin(null),
    })
  }

  const others = (accounts ?? []).filter((gh) => !gh.current)
  return (
    <>
      <DropdownMenu open={open} onOpenChange={setOpen}>
        <DropdownMenuTrigger asChild>
          <AccountTrigger
            login={account?.login}
            showName={showName}
            onPointerEnter={prefetchAccounts}
            onFocus={prefetchAccounts}
          />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-64">
          <CurrentAccount login={account?.login} />
          {isPending || others.length > 0 ? (
            <>
              <DropdownMenuSeparator />
              <DropdownMenuLabel className="text-[11px] tracking-wide text-muted-foreground uppercase">
                Switch account
              </DropdownMenuLabel>
              <AccountItems
                accounts={others}
                loading={isPending}
                disabled={switchAccount.isPending}
                onPick={setPendingLogin}
              />
            </>
          ) : null}
          <DropdownMenuSeparator />
          {vaultId ? <VaultActions vaultId={vaultId} /> : null}
          <DropdownMenuItem
            variant="destructive"
            onSelect={() =>
              logout.mutate(undefined, { onSuccess: () => void navigate({ to: '/sign-in' }) })
            }
          >
            <LogOut /> Sign out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <ConfirmDialog
        open={pendingLogin !== null}
        title={`Switch to ${pendingLogin ?? 'this account'}?`}
        description={`You'll leave ${account?.login ?? 'the current account'} and see ${pendingLogin ?? 'the new account'}'s vaults instead. Anything you haven't saved yet will be discarded.`}
        confirmLabel="Switch account"
        busy={switchAccount.isPending}
        onConfirm={confirmSwitch}
        onCancel={() => setPendingLogin(null)}
      />
    </>
  )
}
