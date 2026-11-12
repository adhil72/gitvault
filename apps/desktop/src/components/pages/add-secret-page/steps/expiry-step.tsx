import type { FormEvent } from 'react'

import { DatePicker } from '@/components/general/date-picker'
import { FormField } from '@/components/general/form-field'
import { SelectField } from '@/components/general/select-field'
import { Textarea } from '@/components/ui/textarea'
import { cn } from '@/lib/utils'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { FIELD } from '../step-field-style'
import { StepActions } from '../step-actions'
import { StepFrame } from '../step-frame'
import { useAddSecretNav } from '../use-add-secret-nav'

const REMINDERS = [
  { value: '3', label: '3 days before' },
  { value: '7', label: '7 days before' },
  { value: '14', label: '14 days before' },
  { value: '30', label: '30 days before' },
]

export function ExpiryStep() {
  const { go } = useAddSecretNav()
  const { expires, remindDays, notes, update } = useAddSecretDraft()

  const submit = (event: FormEvent) => {
    event.preventDefault()
    go('review')
  }

  return (
    <StepFrame
      title="Expiry & notes"
      description="Optional, but recommended for API keys and tokens."
      onSubmit={submit}
      actions={<StepActions onBack={() => go('value')} />}
    >
      <div className={cn('grid gap-5', expires && 'sm:grid-cols-2')}>
        <FormField label="Expires" htmlFor="secret-expires" hint="Leave empty if it never expires">
          <DatePicker
            id="secret-expires"
            value={expires}
            onChange={(value) => update({ expires: value })}
            className={FIELD}
          />
        </FormField>
        {expires ? (
          <FormField label="Remind me" htmlFor="secret-remind" hint="We will flag it in the vault">
            <SelectField
              id="secret-remind"
              value={remindDays}
              onChange={(v) => update({ remindDays: v })}
              options={REMINDERS}
              className={FIELD}
            />
          </FormField>
        ) : null}
      </div>
      <FormField label="Notes" htmlFor="secret-notes">
        <Textarea
          id="secret-notes"
          value={notes}
          onChange={(e) => update({ notes: e.target.value })}
          placeholder="Live key for billing service. Rotate every 90 days."
          className="min-h-28 rounded-xl border-transparent bg-muted/60 px-3.5 py-3 text-[15px] shadow-none hover:bg-muted/90 focus-visible:bg-background dark:bg-muted/60"
        />
      </FormField>
    </StepFrame>
  )
}
