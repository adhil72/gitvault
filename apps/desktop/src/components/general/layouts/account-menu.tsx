import { useNavigate } from '@tanstack/react-router'
import { Check, Lock, LogOut, Settings } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useAccount, useGhAccounts, useLogout, useSwitchAccount } from '@/hooks/use-auth'
import { useLockVault } from '@/hooks/use-vaults'

/** Header menu: switch GitHub CLI account, settings, lock, sign out. */
export function AccountMenu({ vaultId }: { vaultId: string }) {
  const navigate = useNavigate()
  const [open, setOpen] = useState(false)
  const { data: account } = useAccount()
  const { data: accounts = [] } = useGhAccounts(open)
  const switchAccount = useSwitchAccount()
  const logout = useLogout()
  const lock = useLockVault()
  const initial = (account?.login ?? '?').slice(0, 1).toUpperCase()

  const switchTo = (login: string) =>
    switchAccount.mutate(login, {
      // The new account has its own vaults, so go back to the vault picker.
      onSuccess: () => void navigate({ to: '/vaults' }),
      onError: (error) => toast.error(error.message),
    })

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Account menu"
          className="rounded-full border border-done bg-done/10 text-done"
        >
          {initial}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-60">
        <DropdownMenuLabel className="text-[11px] tracking-wide text-muted-foreground uppercase">
          GitHub account
        </DropdownMenuLabel>
        {accounts.map((gh) => (
          <DropdownMenuItem
            key={gh.login}
            disabled={switchAccount.isPending}
            onSelect={() => (gh.current ? undefined : switchTo(gh.login))}
          >
            <span className="flex-1 truncate">{gh.login}</span>
            {gh.current ? <Check className="size-4" /> : null}
          </DropdownMenuItem>
        ))}
        {accounts.length === 0 ? (
          <DropdownMenuLabel className="font-normal">
            {account?.login ?? 'Signed out'}
          </DropdownMenuLabel>
        ) : null}
        <DropdownMenuSeparator />
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
        <DropdownMenuItem
          onSelect={() =>
            logout.mutate(undefined, { onSuccess: () => void navigate({ to: '/sign-in' }) })
          }
        >
          <LogOut /> Sign out
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
