import type { ReactNode } from 'react'

import { AccountMenu } from '@/components/general/layouts/account-menu'
import { Logo } from '@/components/general/logo-mark'

/** The app frame (header + page area) for an account that has no vault to open yet. */
export function NoVaultShell({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-[calc(100vh-var(--titlebar-h))] flex-col">
      <header className="flex h-14 shrink-0 items-center justify-between gap-4 border-b border-border/40 bg-inset px-4">
        <Logo size="sm" />
        <AccountMenu showName />
      </header>
      <main className="flex min-w-0 flex-1 flex-col overflow-auto bg-background">{children}</main>
    </div>
  )
}
