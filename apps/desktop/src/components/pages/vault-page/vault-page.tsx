import { useNavigate } from '@tanstack/react-router'
import { KeyRound, Plus, Upload } from 'lucide-react'

import { EmptyState } from '@/components/general/empty-state'
import { ErrorNotice } from '@/components/general/error-notice'
import { SecretRow } from '@/components/general/secret-row'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useCopySecret } from '@/hooks/use-clipboard'
import { useSecrets } from '@/hooks/use-secrets'
import { useRequireUnlock } from '@/hooks/use-require-unlock'
import { useSettings } from '@/hooks/use-settings'
import type { ApiError } from '@/lib/api-error'

import { buildQuery, hasFilter } from './build-query'
import { LockedNotice } from './locked-notice'
import { RemoteMissingBanner } from './remote-missing-banner'
import { DeleteSecretDialog } from './delete-secret-dialog'
import { EditSecretSheet } from './edit-secret-sheet'
import { ImportEnvDialog } from './import-env-dialog'
import { SecretDetailPanel } from './secret-detail-panel'
import { VaultToolbar } from './vault-toolbar'
import { useVaultNav } from './use-vault-nav'

/** The main screen: list of secrets with filters, and the selected secret's details. */
export function VaultPage() {
  const nav = useVaultNav()
  useSettings()
  const { vaultId, search } = nav
  const { data: secrets, isLoading, error } = useSecrets(buildQuery(vaultId, search))
  const copy = useCopySecret(vaultId)
  const requireUnlock = useRequireUnlock(vaultId)
  const unlockOnly = (error as ApiError | null)?.code === 'E_LOCKED'
  const empty = !isLoading && secrets?.length === 0

  return (
    <div className="flex min-h-0 flex-1">
      <section className="flex min-w-0 flex-1 flex-col overflow-y-auto">
        <VaultToolbar
          count={secrets?.length ?? 0}
          search={search}
          onSort={nav.setSort}
          onImport={() => requireUnlock(() => nav.openPanel('import'))}
        />
        <RemoteMissingBanner vaultId={vaultId} />
        <LockedNotice vaultId={vaultId} nothingReadable={unlockOnly} />
        <ErrorNotice error={unlockOnly ? null : error} />
        {isLoading ? <ListSkeleton /> : null}
        {empty ? (
          <NoSecrets
            vaultId={vaultId}
            filtered={hasFilter(search)}
            onImport={() => requireUnlock(() => nav.openPanel('import'))}
          />
        ) : null}
        <ul className="flex flex-col gap-2 px-8 pb-6">
          {secrets?.map((secret) => (
            <SecretRow
              key={secret.id}
              secret={secret}
              selected={search.secret === secret.id}
              onSelect={() => nav.select(secret.id)}
              onCopy={() => requireUnlock(() => copy.mutate(secret.id))}
            />
          ))}
        </ul>
      </section>
      {search.secret ? (
        <SecretDetailPanel vaultId={vaultId} secretId={search.secret} nav={nav} />
      ) : null}
      <EditSecretSheet nav={nav} />
      <DeleteSecretDialog nav={nav} />
      <ImportEnvDialog nav={nav} />
    </div>
  )
}

function ListSkeleton() {
  return (
    <div className="flex flex-col gap-2 px-8">
      {[0, 1, 2, 3].map((i) => (
        <Skeleton key={i} className="h-[66px] w-full" />
      ))}
    </div>
  )
}

function NoSecrets({
  vaultId,
  filtered,
  onImport,
}: {
  vaultId: string
  filtered: boolean
  onImport: () => void
}) {
  const navigate = useNavigate()
  const requireUnlock = useRequireUnlock(vaultId)
  const goToAddSecret = () =>
    void navigate({ to: '/vaults/$vaultId/secrets/new/$step', params: { vaultId, step: 'type' } })
  if (filtered) {
    return (
      <EmptyState
        icon={KeyRound}
        title="No matching secrets"
        description="Try a different search or filter."
      />
    )
  }
  return (
    <EmptyState
      icon={KeyRound}
      title="No secrets in this vault yet"
      description="Add your first secret or import an existing .env file to get started."
    >
      <Button onClick={() => requireUnlock(goToAddSecret)}>
        <Plus /> Add secret
      </Button>
      <Button variant="ghost" onClick={onImport}>
        <Upload /> Import .env
      </Button>
    </EmptyState>
  )
}
