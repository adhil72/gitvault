import { toast } from 'sonner'

import { ErrorNotice } from '@/components/general/error-notice'
import { Button } from '@/components/ui/button'
import { useCollaborators, useRemoveCollaborator } from '@/hooks/use-collaborators'
import { useVault } from '@/hooks/use-vaults'
import type { CollaboratorDto } from '@/types'

import { SettingsGroup, SettingsRow } from '../settings-row'
import { InviteCollaboratorForm } from './invite-collaborator-form'

/** People GitHub lets into the vault's repository. Only vaults with a repository can share. */
export function CollaboratorsCard({ vaultId }: { vaultId: string }) {
  const { data: vault } = useVault(vaultId)
  if (!vault?.repo) return null
  return <CollaboratorsGroup vaultId={vaultId} />
}

function CollaboratorsGroup({ vaultId }: { vaultId: string }) {
  const { data: people = [], error: loadError } = useCollaborators(vaultId)
  const remove = useRemoveCollaborator(vaultId)
  return (
    <>
      <ErrorNotice error={loadError ?? remove.error} />
      <SettingsGroup
        title="Collaborators"
        footer="Collaborators get access to the vault's GitHub repository. Removing one, or cancelling a pending invitation, revokes that access."
      >
        {people.length === 0 ? (
          <div className="px-5 py-3.5 text-[13px] text-muted-foreground">
            No collaborators yet. Invite someone below.
          </div>
        ) : (
          people.map((person) => (
            <CollaboratorRow
              key={person.login}
              person={person}
              disabled={remove.isPending}
              onRemove={() =>
                remove.mutate(person.login, {
                  onSuccess: () => toast.success(`Removed ${person.login}`),
                })
              }
            />
          ))
        )}
        <InviteCollaboratorForm vaultId={vaultId} />
      </SettingsGroup>
    </>
  )
}

function CollaboratorRow({
  person,
  disabled,
  onRemove,
}: {
  person: CollaboratorDto
  disabled: boolean
  onRemove: () => void
}) {
  const access = person.role === 'write' ? 'Can change secrets' : 'Read only'
  return (
    <SettingsRow
      title={person.login}
      description={person.pending ? `${access} · invitation pending` : access}
    >
      <Button variant="danger" disabled={disabled} onClick={onRemove}>
        {person.pending ? 'Cancel invite' : 'Remove'}
      </Button>
    </SettingsRow>
  )
}
