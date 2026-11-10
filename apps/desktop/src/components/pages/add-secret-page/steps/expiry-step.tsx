import type { FormEvent } from 'react'

import { DatePicker } from '@/components/general/date-picker'
import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { SelectField } from '@/components/general/select-field'
import { WizardActions } from '@/components/general/wizard-actions'
import { Textarea } from '@/components/ui/textarea'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

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
    <form onSubmit={submit} className="flex flex-col gap-5">
      <PageHeading title="Expiry & notes" description="Optional, but recommended for API keys." />
      <FormField label="Expires" htmlFor="secret-expires" hint="Leave empty if it never expires">
        <DatePicker
          id="secret-expires"
          value={expires}
          onChange={(value) => update({ expires: value })}
        />
      </FormField>
      {expires ? (
        <FormField label="Remind me" htmlFor="secret-remind">
          <SelectField
            id="secret-remind"
            value={remindDays}
            onChange={(v) => update({ remindDays: v })}
            options={REMINDERS}
          />
        </FormField>
      ) : null}
      <FormField label="Notes" htmlFor="secret-notes">
        <Textarea
          id="secret-notes"
          value={notes}
          onChange={(e) => update({ notes: e.target.value })}
          placeholder="Live key for billing service. Rotate every 90 days."
          className="min-h-20 bg-inset"
        />
      </FormField>
      <WizardActions onBack={() => go('value')} />
    </form>
  )
}
