import { Link, useParams } from '@tanstack/react-router'
import { GitBranch, Palette, ShieldCheck, SlidersHorizontal, type LucideIcon } from 'lucide-react'

import { cn } from '@/lib/utils'

import { AppearanceTab } from './tabs/appearance-tab'
import { GeneralTab } from './tabs/general-tab'
import { GithubSyncTab } from './tabs/github-sync-tab'
import { SecurityTab } from './tabs/security-tab'

const TABS: { id: string; label: string; icon: LucideIcon }[] = [
  { id: 'general', label: 'General', icon: SlidersHorizontal },
  { id: 'security', label: 'Security', icon: ShieldCheck },
  { id: 'github-sync', label: 'GitHub & Sync', icon: GitBranch },
  { id: 'appearance', label: 'Appearance', icon: Palette },
]

function TabContent({ tab, vaultId }: { tab: string; vaultId: string }) {
  if (tab === 'security') return <SecurityTab vaultId={vaultId} />
  if (tab === 'github-sync') return <GithubSyncTab vaultId={vaultId} />
  if (tab === 'appearance') return <AppearanceTab />
  return <GeneralTab />
}

export function SettingsPage() {
  const { vaultId, tab } = useParams({ from: '/_authed/vaults/$vaultId/settings/$tab' })
  return (
    <div className="flex min-h-0 flex-1">
      <nav className="flex w-56 shrink-0 flex-col gap-0.5 p-3 pt-6" aria-label="Settings">
        <h1 className="px-3 pb-3 text-[22px] font-semibold tracking-[-0.02em]">Settings</h1>
        {TABS.map(({ id, label, icon: Icon }) => (
          <Link
            key={id}
            to="/vaults/$vaultId/settings/$tab"
            params={{ vaultId, tab: id }}
            className={cn(
              'flex h-9 items-center gap-2.5 rounded-lg px-3 text-[13px] outline-none transition-[background-color,transform] duration-150 ease-(--ease-settle) focus-visible:ring-2 focus-visible:ring-ring/60 active:scale-[0.98] motion-reduce:active:scale-100',
              tab === id
                ? 'bg-muted font-medium text-foreground'
                : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground',
            )}
          >
            <Icon className={cn('size-4', tab === id && 'text-info')} />
            {label}
          </Link>
        ))}
      </nav>
      <div className="min-w-0 flex-1 overflow-y-auto">
        {/* key restarts the rise-in so switching tabs feels like one continuous surface changing. */}
        <div
          key={tab}
          className="mx-auto flex w-full max-w-2xl animate-rise flex-col gap-6 px-8 pt-8 pb-16 motion-reduce:animate-none"
        >
          <TabContent tab={tab} vaultId={vaultId} />
        </div>
      </div>
    </div>
  )
}
