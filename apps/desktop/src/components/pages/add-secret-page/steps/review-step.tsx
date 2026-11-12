import { GitCommitHorizontal, Pencil } from 'lucide-react'
import type { ReactNode } from 'react'

import { EnvBadge } from '@/components/general/env-badge'
import { ErrorNotice } from '@/components/general/error-notice'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { AddSecretStep } from '@/config/wizards'
import { useAddSecret } from '@/hooks/use-secrets'
import { fromDateInput } from '@/lib/format'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { StepActions } from '../step-actions'
import { StepFrame } from '../step-frame'
import { useAddSecretNav } from '../use-add-secret-nav'

function parseTags(text: string): string[] {
  return text
    .split(',')
    .map((t) => t.trim())
    .filter(Boolean)
}

const EMPTY = <span className="text-muted-foreground">Not set</span>

interface ReviewRowProps {
  label: string
  step: AddSecretStep
  onEdit: (step: AddSecretStep) => void
  children: ReactNode
}

function ReviewRow({ label, step, onEdit, children }: ReviewRowProps) {
  return (
    <div className="flex items-center gap-4 py-3.5 text-sm">
      <dt className="w-28 shrink-0 text-muted-foreground">{label}</dt>
      <dd className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5 font-medium">{children}</dd>
      <Button
        type="button"
        variant="ghost"
        size="xs"
        aria-label={`Edit ${label.toLowerCase()}`}
        onClick={() => onEdit(step)}
      >
        <Pencil /> Edit
      </Button>
    </div>
  )
}

export function ReviewStep() {
  const { vaultId, go } = useAddSecretNav()
  const draft = useAddSecretDraft()
  const add = useAddSecret(vaultId)
  const tags = parseTags(draft.tags)

  const save = () => {
    const expires = fromDateInput(draft.expires)
    add.mutate(
      {
        vault_id: vaultId,
        name: draft.name,
        kind: draft.kind,
        value: draft.value,
        environment: draft.environment || null,
        tags,
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

  return (
    <StepFrame
      title="Review"
      description="Check everything before it is encrypted and committed."
      actions={
        <StepActions
          nextType="button"
          nextLabel="Save secret"
          nextIcon={null}
          busy={add.isPending}
          onNext={save}
          onBack={() => go('expiry')}
        />
      }
    >
      <dl className="divide-y divide-border border-y border-border">
        <ReviewRow label="Name" step="details" onEdit={go}>
          <span className="truncate font-mono">{draft.name}</span>
        </ReviewRow>
        <ReviewRow label="Environment" step="details" onEdit={go}>
          {draft.environment ? <EnvBadge name={draft.environment} /> : EMPTY}
        </ReviewRow>
        <ReviewRow label="Tags" step="details" onEdit={go}>
          {tags.length > 0
            ? tags.map((tag) => (
                <Badge key={tag} variant="secondary">
                  {tag}
                </Badge>
              ))
            : EMPTY}
        </ReviewRow>
        <ReviewRow label="Value" step="value" onEdit={go}>
          <span className="font-mono tracking-wider">{'••••••••' + draft.value.slice(-4)}</span>
        </ReviewRow>
        <ReviewRow label="Expires" step="expiry" onEdit={go}>
          {draft.expires || <span className="text-muted-foreground">Never</span>}
        </ReviewRow>
        <ReviewRow label="Notes" step="expiry" onEdit={go}>
          {draft.notes ? <span className="line-clamp-2 font-normal">{draft.notes}</span> : EMPTY}
        </ReviewRow>
      </dl>
      <p className="flex items-center gap-2 text-[13px] text-muted-foreground">
        <GitCommitHorizontal className="size-4 shrink-0" />
        Saving pushes an encrypted commit to your vault repository.
      </p>
      <ErrorNotice error={add.error} />
    </StepFrame>
  )
}
