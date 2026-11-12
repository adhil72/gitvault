import { Eye, EyeOff, ShieldCheck, Sparkles } from 'lucide-react'
import { useState, type FormEvent } from 'react'

import { FormField } from '@/components/general/form-field'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { useGenerate } from '@/hooks/use-tools'
import { cn } from '@/lib/utils'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { StepActions } from '../step-actions'
import { StepFrame } from '../step-frame'
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
    <StepFrame
      title="Secret value"
      description="Paste the value, or let GitVault generate a strong one."
      onSubmit={submit}
      actions={<StepActions onBack={() => go('details')} nextDisabled={!value} />}
    >
      <div className="flex flex-col gap-3">
        <FormField label="Value" htmlFor="secret-value">
          <Textarea
            id="secret-value"
            autoFocus
            value={value}
            onChange={(e) => update({ value: e.target.value })}
            placeholder="Paste the value or generate one"
            className={cn(
              'min-h-36 resize-none rounded-xl border-transparent bg-muted/60 px-4 py-3.5 font-mono text-[13px] shadow-none hover:bg-muted/90 focus-visible:bg-background dark:bg-muted/60',
              !visible && '[-webkit-text-security:disc]',
            )}
            spellCheck={false}
            autoComplete="off"
          />
        </FormField>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="secondary"
            size="sm"
            className="rounded-full"
            disabled={generate.isPending}
            onClick={() =>
              generate.mutate(GENERATE_DEFAULTS, { onSuccess: (g) => update({ value: g.value }) })
            }
          >
            <Sparkles /> Generate
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="rounded-full text-muted-foreground"
            onClick={() => setVisible((v) => !v)}
          >
            {visible ? <EyeOff /> : <Eye />} {visible ? 'Hide' : 'Reveal'}
          </Button>
        </div>
      </div>
      <p className="flex items-center gap-2 text-[13px] text-muted-foreground">
        <ShieldCheck className="size-4 shrink-0" />
        Encrypted on this device. Only ciphertext reaches GitHub.
      </p>
    </StepFrame>
  )
}
