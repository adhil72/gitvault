import { useNavigate } from '@tanstack/react-router'
import { toast } from 'sonner'

import { ErrorNotice } from '@/components/general/error-notice'
import { GithubMark } from '@/components/general/github-mark'
import { PageHeading } from '@/components/general/page-heading'
import { Button } from '@/components/ui/button'
import { useAccount, useLogout } from '@/hooks/use-auth'
import { useConflicts, useResolveConflict, useSyncNow, useSyncStatus } from '@/hooks/use-sync'
import { useVault } from '@/hooks/use-vaults'
import { timeAgo } from '@/lib/format'

import { SettingsCard, SettingsRow } from '../settings-row'

export function GithubSyncTab({ vaultId }: { vaultId: string }) {
  return (
    <>
      <PageHeading
        title="GitHub & Sync"
        description="Your account identifies the vault. Secrets sync as encrypted commits."
      />
      <AccountCard />
      <RepositoryCard vaultId={vaultId} />
      <Conflicts vaultId={vaultId} />
    </>
  )
}

function AccountCard() {
  const navigate = useNavigate()
  const { data: account } = useAccount()
  const logout = useLogout()
  return (
    <section className="flex items-center gap-4 rounded-lg border border-border bg-card p-5">
      <span className="flex size-12 items-center justify-center rounded-full border border-done bg-done/10 text-lg font-bold text-done">
        {(account?.login ?? '?').slice(0, 1).toUpperCase()}
      </span>
      <div className="flex flex-1 flex-col gap-0.5">
        <span className="font-semibold">{account?.login}</span>
        <span className="flex items-center gap-1.5 text-[13px] text-muted-foreground">
          <GithubMark className="size-3.5" /> Signed in with GitHub
        </span>
      </div>
      <Button
        variant="danger"
        onClick={() =>
          logout.mutate(undefined, { onSuccess: () => void navigate({ to: '/sign-in' }) })
        }
      >
        Sign out
      </Button>
    </section>
  )
}

function RepositoryCard({ vaultId }: { vaultId: string }) {
  const { data: vault } = useVault(vaultId)
  const { data: status } = useSyncStatus(vaultId)
  const sync = useSyncNow(vaultId)
  const lastSync = status?.last_sync ? timeAgo(status.last_sync) : 'Never'
  const runSync = () =>
    sync.mutate(undefined, {
      onSuccess: (r) => toast.success(`Synced: ${r.pulled} pulled, ${r.pushed} pushed`),
      onError: (error) => toast.error(error.message),
    })
  return (
    <SettingsCard>
      <SettingsRow
        title="Repository"
        description={vault?.repo ?? 'This vault has no GitHub repository (local only).'}
      >
        <span className="rounded-full border border-success/60 bg-success/10 px-2.5 py-0.5 text-xs text-success">
          {vault?.repo ? 'Private' : 'Local'}
        </span>
      </SettingsRow>
      <SettingsRow
        title="Last sync"
        description={`${lastSync} · ${status?.pending ?? 0} pending changes`}
      >
        <Button
          variant="secondary"
          disabled={sync.isPending || status?.state === 'no_remote'}
          onClick={runSync}
        >
          {sync.isPending ? 'Syncing…' : 'Sync now'}
        </Button>
      </SettingsRow>
    </SettingsCard>
  )
}

function Conflicts({ vaultId }: { vaultId: string }) {
  const { data: conflicts } = useConflicts(vaultId)
  const resolve = useResolveConflict(vaultId)
  if (!conflicts?.length) return null
  return (
    <section className="flex flex-col gap-3 rounded-lg border border-destructive/50 bg-destructive/5 p-5">
      <h2 className="font-semibold">Conflicts</h2>
      <p className="text-[13px] text-muted-foreground">
        These secrets were changed on two devices. Choose which version to keep.
      </p>
      <ErrorNotice error={resolve.error} />
      {conflicts.map((c) => (
        <div key={c.secret_id} className="flex items-center gap-3">
          <span className="flex-1 truncate font-mono text-sm">{c.name ?? c.secret_id}</span>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => resolve.mutate({ secretId: c.secret_id, keep: 'local' })}
          >
            Keep mine
          </Button>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => resolve.mutate({ secretId: c.secret_id, keep: 'remote' })}
          >
            Keep GitHub&apos;s
          </Button>
        </div>
      ))}
    </section>
  )
}
