import { useNavigate, useSearch } from '@tanstack/react-router'
import { Database, Plus } from 'lucide-react'
import { useEffect } from 'react'

import { ErrorNotice } from '@/components/general/error-notice'
import { PageHeading } from '@/components/general/page-heading'
import { Skeleton } from '@/components/ui/skeleton'
import { useRemoteVaults, useVaults } from '@/hooks/use-vaults'

import { VaultCard } from './vault-card'

/** Lists local and GitHub vaults. Jumps straight into the current vault unless `?all=1`. */
export function VaultsHomePage() {
  const navigate = useNavigate()
  const { all } = useSearch({ from: '/_authed/vaults/' })
  const { data, isLoading, error } = useVaults()
  const remote = useRemoteVaults(Boolean(all ?? (data && data.vaults.length === 0)))

  useEffect(() => {
    if (!data || all) return
    const current = data.vaults.find((v) => v.current && v.id)
    if (current?.id) {
      void navigate({
        to: current.unlocked ? '/vaults/$vaultId' : '/vaults/$vaultId/unlock',
        params: { vaultId: current.id },
        search: {},
        replace: true,
      })
    } else if (data.vaults.length === 0) {
      void navigate({ to: '/vaults/new/$step', params: { step: 'name' }, replace: true })
    }
  }, [data, all, navigate])

  return (
    <main className="mx-auto flex min-h-[calc(100vh-var(--titlebar-h))] w-full max-w-xl flex-col justify-center gap-6 p-8">
      <PageHeading
        title="Your vaults"
        description="Open a vault, connect one from GitHub, or create a new one."
      />
      {isLoading ? <Skeleton className="h-40 w-full" /> : null}
      <ErrorNotice
        error={
          error ??
          (remote.data?.remote_error ? new Error(`GitHub: ${remote.data.remote_error}`) : null)
        }
      />
      <ul className="flex flex-col gap-3">
        {[...(data?.vaults ?? []), ...(remote.data?.vaults ?? [])].map((vault) => (
          <VaultCard key={vault.id ?? vault.repo} vault={vault} />
        ))}
        <li>
          <button
            type="button"
            onClick={() => void navigate({ to: '/vaults/new/$step', params: { step: 'name' } })}
            className="flex w-full items-center gap-3.5 rounded-lg border border-dashed border-input p-4 text-left hover:bg-muted/40"
          >
            <span className="flex size-9 items-center justify-center rounded-lg bg-success/15 text-success">
              <Plus className="size-[18px]" />
            </span>
            <span className="flex flex-col gap-0.5">
              <span className="font-semibold">Create a new vault</span>
              <span className="text-xs text-muted-foreground">
                Creates a private repository in your GitHub account
              </span>
            </span>
          </button>
        </li>
      </ul>
      <Database className="hidden" />
    </main>
  )
}
