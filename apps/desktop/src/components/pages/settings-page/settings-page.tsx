import { Link, useParams } from '@tanstack/react-router'

import { cn } from '@/lib/utils'

import { AppearanceTab } from './tabs/appearance-tab'
import { GeneralTab } from './tabs/general-tab'
import { GithubSyncTab } from './tabs/github-sync-tab'
import { SecurityTab } from './tabs/security-tab'

const TABS = [
  { id: 'general', label: 'General' },
  { id: 'security', label: 'Security' },
  { id: 'github-sync', label: 'GitHub & Sync' },
  { id: 'appearance', label: 'Appearance' },
] as const

export function SettingsPage() {
  const { vaultId, tab } = useParams({ from: '/_authed/vaults/$vaultId/settings/$tab' })
  return (
    <div className="flex min-h-0 flex-1">
      <nav
        className="flex w-60 shrink-0 flex-col gap-1 border-r border-border p-5"
        aria-label="Settings"
      >
        <h1 className="pb-3 text-xl font-bold">Settings</h1>
        {TABS.map(({ id, label }) => (
          <Link
            key={id}
            to="/vaults/$vaultId/settings/$tab"
            params={{ vaultId, tab: id }}
            className={cn(
              'rounded-md px-3 py-2 text-sm',
              tab === id ? 'bg-muted font-semibold' : 'hover:bg-muted/50',
            )}
          >
            {label}
          </Link>
        ))}
      </nav>
      <div className="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto p-8">
        {tab === 'security' ? <SecurityTab vaultId={vaultId} /> : null}
        {tab === 'github-sync' ? <GithubSyncTab vaultId={vaultId} /> : null}
        {tab === 'appearance' ? <AppearanceTab /> : null}
        {!['security', 'github-sync', 'appearance'].includes(tab) ? <GeneralTab /> : null}
      </div>
    </div>
  )
}
