import { Link } from '@tanstack/react-router'
import { KeyRound, Plus, Upload } from 'lucide-react'

import { EmptyState } from '@/components/general/empty-state'
import { ErrorNotice } from '@/components/general/error-notice'
import { SecretRow } from '@/components/general/secret-row'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useCopySecret } from '@/hooks/use-clipboard'
import { useSecrets } from '@/hooks/use-secrets'
import { useSettings } from '@/hooks/use-settings'

import { buildQuery, hasFilter } from './build-query'
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
  const empty = !isLoading && secrets?.length === 0

  return (
    <div className="flex min-h-0 flex-1">
      <section className="flex min-w-0 flex-1 flex-col overflow-y-auto">
        <VaultToolbar
          count={secrets?.length ?? 0}
          search={search}
          onSort={nav.setSort}
          onImport={() => nav.openPanel('import')}
        />
        <ErrorNotice error={error} />
        {isLoading ? <ListSkeleton /> : null}
        {empty ? (
          <NoSecrets
            vaultId={vaultId}
            filtered={hasFilter(search)}
            onImport={() => nav.openPanel('import')}
          />
        ) : null}
        <ul className="flex flex-col gap-2 px-6 pb-6">
          {secrets?.map((secret) => (
            <SecretRow
              key={secret.id}
              secret={secret}
              selected={search.secret === secret.id}
              onSelect={() => nav.select(secret.id)}
              onCopy={() => copy.mutate(secret.id)}
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
    <div className="flex flex-col gap-2 px-6">
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
      <Button asChild>
        <Link to="/vaults/$vaultId/secrets/new/$step" params={{ vaultId, step: 'type' }}>
          <Plus /> Add secret
        </Link>
      </Button>
      <Button variant="secondary" onClick={onImport}>
        <Upload /> Import .env
      </Button>
    </EmptyState>
  )
}
