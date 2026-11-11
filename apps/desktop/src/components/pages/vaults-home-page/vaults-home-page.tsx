import { useNavigate } from '@tanstack/react-router'
import { Plus } from 'lucide-react'

import { ErrorNotice } from '@/components/general/error-notice'
import { PageHeading } from '@/components/general/page-heading'
import { Skeleton } from '@/components/ui/skeleton'
import { useRemoteVaults, useVaults } from '@/hooks/use-vaults'

import { VaultCard } from './vault-card'

/** The first screen after opening the app: pick a vault (local or on GitHub) or create one. */
export function VaultsHomePage() {
  const navigate = useNavigate()
  const { data, isLoading, error } = useVaults()
  const remote = useRemoteVaults(true)

  return (
    <main className="mx-auto flex min-h-[calc(100vh-var(--titlebar-h))] w-full max-w-xl flex-col justify-center gap-6 p-8">
      <PageHeading
        title="Your vaults"
        description="Select a vault to open. Secrets stay locked until you enter the master password."
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
    </main>
  )
}
