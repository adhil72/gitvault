import { Info } from 'lucide-react'

import { Callout } from '@/components/general/callout'
import { ErrorNotice } from '@/components/general/error-notice'
import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { useAddSecret } from '@/hooks/use-secrets'
import { fromDateInput } from '@/lib/format'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { useAddSecretNav } from '../use-add-secret-nav'

const KIND_LABELS = {
  api_key: 'API key',
  password: 'Password',
  certificate: 'Certificate',
  note: 'Note',
  env_file: '.env file',
} as const

function parseTags(text: string): string[] {
  return text
    .split(',')
    .map((t) => t.trim())
    .filter(Boolean)
}

export function ReviewStep() {
  const { vaultId, go } = useAddSecretNav()
  const draft = useAddSecretDraft()
  const add = useAddSecret(vaultId)

  const save = () => {
    const expires = fromDateInput(draft.expires)
    add.mutate(
      {
        vault_id: vaultId,
        name: draft.name,
        kind: draft.kind,
        value: draft.value,
        environment: draft.environment || null,
        tags: parseTags(draft.tags),
        notes: draft.notes || null,
        expires_at: expires,
        remind_before_days: expires === null ? null : Number(draft.remindDays),
      },
      {
        onSuccess: (saved) => {
          draft.update({ savedName: saved.name, savedId: saved.id, value: '' })
          go('saved')
        },
      },
    )
  }

  const rows: [string, string][] = [
    ['Type', KIND_LABELS[draft.kind]],
    ['Name', draft.name],
    ['Environment', draft.environment || '-'],
    ['Tags', parseTags(draft.tags).join(', ') || '-'],
    ['Value', '••••••••' + draft.value.slice(-4)],
    ['Expires', draft.expires || 'Never'],
  ]

  return (
    <div className="flex flex-col gap-5">
      <PageHeading title="Review" description="Everything look right?" />
      <dl className="flex flex-col rounded-lg border border-border bg-card">
        {rows.map(([label, value], index) => (
          <div
            key={label}
            className={`flex gap-3 px-4 py-2.5 text-[13px] ${index < rows.length - 1 ? 'border-b border-border' : ''}`}
          >
            <dt className="w-24 shrink-0 text-muted-foreground">{label}</dt>
            <dd className="min-w-0 flex-1 truncate font-medium">{value}</dd>
          </div>
        ))}
      </dl>
      <Callout tone="info" icon={<Info className="size-4 text-info" />}>
        Saving pushes an encrypted commit to your vault repository.
      </Callout>
      <ErrorNotice error={add.error} />
      <WizardActions
        nextType="button"
        nextLabel="Save secret"
        busy={add.isPending}
        onNext={save}
        onBack={() => go('expiry')}
      />
    </div>
  )
}
