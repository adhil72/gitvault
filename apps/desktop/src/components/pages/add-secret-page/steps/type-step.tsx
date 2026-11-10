import { FileText, KeyRound, Lock, ScrollText, Terminal, type LucideIcon } from 'lucide-react'
import type { FormEvent } from 'react'

import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { cn } from '@/lib/utils'
import { useAddSecretDraft } from '@/stores/add-secret-draft'
import type { SecretKind } from '@/types'

import { useAddSecretNav } from '../use-add-secret-nav'

const KINDS: { kind: SecretKind; title: string; hint: string; icon: LucideIcon }[] = [
  { kind: 'api_key', title: 'API key', hint: 'Tokens and keys for services', icon: KeyRound },
  { kind: 'password', title: 'Password', hint: 'Logins and database passwords', icon: Lock },
  {
    kind: 'certificate',
    title: 'Certificate / key file',
    hint: 'PEM, PFX and SSH keys',
    icon: FileText,
  },
  { kind: 'note', title: 'Secure note', hint: 'Free-form encrypted text', icon: ScrollText },
  { kind: 'env_file', title: '.env file', hint: 'Several variables at once', icon: Terminal },
]

export function TypeStep() {
  const { go, backToVault } = useAddSecretNav()
  const { kind, update, reset } = useAddSecretDraft()

  const submit = (event: FormEvent) => {
    event.preventDefault()
    go('details')
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-5">
      <PageHeading title="Choose a secret type" description="You can change this later." />
      <RadioGroup
        value={kind}
        onValueChange={(v) => update({ kind: v as SecretKind })}
        className="gap-2.5"
      >
        {KINDS.map(({ kind: value, title, hint, icon: Icon }) => (
          <label
            key={value}
            htmlFor={`kind-${value}`}
            className={cn(
              'flex cursor-pointer items-center gap-3 rounded-lg border px-3.5 py-2.5',
              kind === value ? 'border-info bg-info/10' : 'border-border bg-card',
            )}
          >
            <span className="flex size-8 items-center justify-center rounded-lg bg-muted">
              <Icon className="size-4" />
            </span>
            <span className="flex flex-1 flex-col">
              <span className="text-sm font-semibold">{title}</span>
              <span className="text-xs text-muted-foreground">{hint}</span>
            </span>
            <RadioGroupItem id={`kind-${value}`} value={value} />
          </label>
        ))}
      </RadioGroup>
      <WizardActions
        backLabel="Cancel"
        onBack={() => {
          reset()
          backToVault()
        }}
      />
    </form>
  )
}
