import { Link, useSearch } from '@tanstack/react-router'
import { Clock, History, KeyRound, SlidersHorizontal, Star, Wand2 } from 'lucide-react'
import type { ReactNode } from 'react'

import { envDotClass } from '@/components/general/env-badge'
import { useFacets } from '@/hooks/use-secrets'
import { cn } from '@/lib/utils'
import type { VaultSearch } from '@/schemas/vault-search'
import { useSidebar } from '@/stores/sidebar'

const NAV_LINK =
  'block rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-ring/60 active:scale-[0.98] motion-reduce:active:scale-100 transition-transform duration-150 ease-(--ease-settle)'

interface RowProps {
  label: string
  count?: number | undefined
  icon?: ReactNode
  active: boolean
}

/** A zero count is noise, so only show counts that tell you something. */
function Row({ label, count, icon, active }: RowProps) {
  const collapsed = useSidebar((s) => s.collapsed)
  return (
    <span
      title={collapsed ? label : undefined}
      className={cn(
        'flex h-9 min-w-[224px] items-center gap-2.5 rounded-lg px-3 text-[13px] transition-colors duration-150',
        active
          ? 'bg-sidebar-accent font-medium text-foreground'
          : 'text-muted-foreground hover:bg-sidebar-accent/60 hover:text-foreground',
      )}
    >
      <span className={cn('flex size-4 items-center justify-center', active && 'text-info')}>
        {icon}
      </span>
      <span className="flex-1 truncate transition-opacity duration-200 group-data-[rail]/nav:opacity-0">
        {label}
      </span>
      {count ? (
        <span className="text-xs text-muted-foreground tabular-nums transition-opacity duration-200 group-data-[rail]/nav:opacity-0">
          {count}
        </span>
      ) : null}
    </span>
  )
}

function SectionTitle({ children }: { children: ReactNode }) {
  return (
    <h3 className="min-w-[224px] px-3 pt-5 pb-1.5 text-xs transition-opacity duration-200 group-data-[rail]/nav:opacity-0 font-medium tracking-[0.01em] text-muted-foreground/80">
      {children}
    </h3>
  )
}

export function AppSidebar({ vaultId }: { vaultId: string }) {
  const { data } = useFacets(vaultId)
  const search = useSearch({ strict: false }) as VaultSearch
  const collapsed = useSidebar((s) => s.collapsed)
  const none = !search.environment && !search.view && !search.tag
  const link = (next: VaultSearch) => ({
    to: '/vaults/$vaultId' as const,
    params: { vaultId },
    search: next,
    className: NAV_LINK,
  })
  return (
    <nav
      aria-label="Vault"
      data-rail={collapsed ? '' : undefined}
      className="group/nav flex w-[240px] shrink-0 flex-col justify-between overflow-x-hidden overflow-y-auto border-r border-border/40 bg-sidebar p-2 transition-[width] duration-300 ease-(--ease-settle) data-[rail]:w-14 motion-reduce:transition-none"
    >
      <div className="flex flex-col gap-0.5">
        <Link {...link({})}>
          <Row
            label="All secrets"
            count={data?.total}
            icon={<KeyRound className="size-4" />}
            active={none}
          />
        </Link>
        <Link {...link({ view: 'favorites' })}>
          <Row
            label="Favorites"
            count={data?.favorites}
            icon={<Star className="size-4" />}
            active={search.view === 'favorites'}
          />
        </Link>
        <Link {...link({ view: 'expiring' })}>
          <Row
            label="Expiring soon"
            count={data?.expiring_soon}
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
      <div className="flex flex-col gap-0.5">
        <Link to="/vaults/$vaultId/tools/generator" params={{ vaultId }} className={NAV_LINK}>
          {({ isActive }) => (
            <Row label="Password generator" icon={<Wand2 className="size-4" />} active={isActive} />
          )}
        </Link>
        <Link to="/vaults/$vaultId/audit" params={{ vaultId }} className={NAV_LINK}>
          {({ isActive }) => (
            <Row label="Audit log" icon={<History className="size-4" />} active={isActive} />
          )}
        </Link>
        <Link
          to="/vaults/$vaultId/settings/$tab"
          params={{ vaultId, tab: 'general' }}
          className={NAV_LINK}
        >
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
