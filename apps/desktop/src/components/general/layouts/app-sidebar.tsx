import { Link, useSearch } from '@tanstack/react-router'
import { Clock, History, KeyRound, SlidersHorizontal, Star, Wand2 } from 'lucide-react'
import type { ReactNode } from 'react'

import { envDotClass } from '@/components/general/env-badge'
import { useFacets } from '@/hooks/use-secrets'
import { cn } from '@/lib/utils'
import type { VaultSearch } from '@/schemas/vault-search'

interface RowProps {
  label: string
  count?: number
  icon?: ReactNode
  active: boolean
}

function Row({ label, count, icon, active }: RowProps) {
  return (
    <span
      className={cn(
        'relative flex items-center gap-2.5 px-4 py-2 text-sm',
        active ? 'bg-subtle font-semibold' : 'hover:bg-subtle/60',
      )}
    >
      {active ? <span className="absolute top-2 left-0 h-5 w-[3px] rounded bg-info" /> : null}
      {icon}
      <span className="flex-1 truncate">{label}</span>
      {count !== undefined ? (
        <span className="font-mono text-xs text-muted-foreground">{count}</span>
      ) : null}
    </span>
  )
}

function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <h3 className="border-t border-sidebar-border px-4 pt-4 pb-1.5 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase first:border-t-0">
      {children}
    </h3>
  )
}

export function AppSidebar({ vaultId }: { vaultId: string }) {
  const { data } = useFacets(vaultId)
  const search = useSearch({ strict: false }) as VaultSearch
  const none = !search.environment && !search.view && !search.tag
  const link = (next: VaultSearch) => ({
    to: '/vaults/$vaultId' as const,
    params: { vaultId },
    search: next,
  })
  return (
    <nav className="flex w-[300px] shrink-0 flex-col justify-between overflow-y-auto border-r border-sidebar-border bg-sidebar">
      <div className="flex flex-col py-2">
        <Link {...link({})}>
          <Row
            label="All secrets"
            count={data?.total ?? 0}
            icon={<KeyRound className="size-4" />}
            active={none}
          />
        </Link>
        <Link {...link({ view: 'favorites' })}>
          <Row
            label="Favorites"
            count={data?.favorites ?? 0}
            icon={<Star className="size-4" />}
            active={search.view === 'favorites'}
          />
        </Link>
        <Link {...link({ view: 'expiring' })}>
          <Row
            label="Expiring soon"
            count={data?.expiring_soon ?? 0}
            icon={<Clock className="size-4" />}
            active={search.view === 'expiring'}
          />
        </Link>
        <SectionTitle>Environments</SectionTitle>
        {data?.environments.map((e) => (
          <Link key={e.name} {...link({ environment: e.name })}>
            <Row
              label={e.name}
              count={e.count}
              active={search.environment === e.name}
              icon={<span className={cn('size-2 rounded-full', envDotClass(e.name))} />}
            />
          </Link>
        ))}
      </div>
      <div className="flex flex-col border-t border-sidebar-border py-2">
        <Link to="/vaults/$vaultId/tools/generator" params={{ vaultId }}>
          {({ isActive }) => (
            <Row label="Password generator" icon={<Wand2 className="size-4" />} active={isActive} />
          )}
        </Link>
        <Link to="/vaults/$vaultId/audit" params={{ vaultId }}>
          {({ isActive }) => (
            <Row label="Audit log" icon={<History className="size-4" />} active={isActive} />
          )}
        </Link>
        <Link to="/vaults/$vaultId/settings/$tab" params={{ vaultId, tab: 'general' }}>
          {({ isActive }) => (
            <Row
              label="Settings"
              icon={<SlidersHorizontal className="size-4" />}
              active={isActive}
            />
          )}
        </Link>
      </div>
    </nav>
  )
}
