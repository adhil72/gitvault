import { Eye, EyeOff, Info, RefreshCw } from 'lucide-react'
import { useState, type FormEvent } from 'react'

import { Callout } from '@/components/general/callout'
import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useGenerate } from '@/hooks/use-tools'
import { cn } from '@/lib/utils'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { useAddSecretNav } from '../use-add-secret-nav'

const GENERATE_DEFAULTS = {
  kind: 'password',
  length: 32,
  uppercase: true,
  lowercase: true,
  digits: true,
  symbols: false,
  avoid_ambiguous: true,
  words: 5,
  separator: '-',
  capitalize: false,
} as const

export function ValueStep() {
  const { go } = useAddSecretNav()
  const { value, update } = useAddSecretDraft()
  const [visible, setVisible] = useState(true)
  const generate = useGenerate()

  const submit = (event: FormEvent) => {
    event.preventDefault()
    go('expiry')
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-5">
      <PageHeading title="Secret value" description="The value is hidden by default." />
      <FormField label="Value" htmlFor="secret-value">
        <Textarea
          id="secret-value"
          autoFocus
          value={value}
          onChange={(e) => update({ value: e.target.value })}
          placeholder="Paste the value or generate one"
          className={cn(
            'min-h-24 bg-inset font-mono text-[13px]',
            !visible && 'text-security-disc [-webkit-text-security:disc]',
          )}
          spellCheck={false}
          autoComplete="off"
        />
      </FormField>
      <div className="flex items-center justify-between">
        <Button
          type="button"
          variant="link"
          size="sm"
          className="px-0"
          disabled={generate.isPending}
          onClick={() =>
            generate.mutate(GENERATE_DEFAULTS, { onSuccess: (g) => update({ value: g.value }) })
          }
        >
          <RefreshCw /> Generate instead
        </Button>
        <Button type="button" variant="ghost" size="sm" onClick={() => setVisible((v) => !v)}>
          {visible ? <EyeOff /> : <Eye />} {visible ? 'Hide' : 'Reveal'}
        </Button>
      </div>
      <Callout tone="info" icon={<Info className="size-4 text-info" />}>
        Values are encrypted before they leave this device. Only ciphertext reaches GitHub.
      </Callout>
      <WizardActions onBack={() => go('details')} nextDisabled={!value} />
    </form>
  )
}
