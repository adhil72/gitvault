import { Outlet, useParams } from '@tanstack/react-router'

import { UnlockDialog } from '@/components/general/unlock-dialog'

import { AppHeader } from './app-header'
import { AppSidebar } from './app-sidebar'

/** Header, sidebar and the current page for everything inside a vault (locked or not). */
export function AppShell() {
  const { vaultId } = useParams({ from: '/_authed/vaults/$vaultId' })
  return (
    <div className="flex h-[calc(100vh-var(--titlebar-h))] flex-col">
      <AppHeader vaultId={vaultId} />
      <div className="flex min-h-0 flex-1">
        <AppSidebar vaultId={vaultId} />
        <main className="flex min-w-0 flex-1 flex-col overflow-auto bg-background">
          <Outlet />
        </main>
      </div>
      <UnlockDialog vaultId={vaultId} />
    </div>
  )
}
