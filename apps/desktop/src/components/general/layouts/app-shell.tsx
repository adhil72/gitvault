import { Outlet, useNavigate, useParams } from '@tanstack/react-router'
import { useEffect } from 'react'

import { useVault } from '@/hooks/use-vaults'

import { AppHeader } from './app-header'
import { AppSidebar } from './app-sidebar'

/** Header, sidebar and the current page for everything inside an unlocked vault. */
export function AppShell() {
  const { vaultId } = useParams({ from: '/_authed/vaults/$vaultId' })
  const navigate = useNavigate()
  const { data: vault } = useVault(vaultId)

  // The backend locks the vault after the idle timeout; send the user to the unlock page.
  useEffect(() => {
    if (vault && !vault.unlocked) {
      void navigate({ to: '/vaults/$vaultId/unlock', params: { vaultId }, search: {} })
    }
  }, [vault, vaultId, navigate])

  return (
    <div className="flex h-[calc(100vh-var(--titlebar-h))] flex-col">
      <AppHeader vaultId={vaultId} />
      <div className="flex min-h-0 flex-1">
        <AppSidebar vaultId={vaultId} />
        <main className="flex min-w-0 flex-1 flex-col overflow-auto bg-background">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
