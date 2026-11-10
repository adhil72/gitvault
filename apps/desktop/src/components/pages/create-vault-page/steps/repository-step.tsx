import { useNavigate } from '@tanstack/react-router'
import { Lock } from 'lucide-react'
import { useState, type FormEvent } from 'react'

import { Callout } from '@/components/general/callout'
import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { Input } from '@/components/ui/input'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { useAccount } from '@/hooks/use-auth'
import { defaultRepoName } from '@/lib/slug'
import { cn } from '@/lib/utils'
import { useCreateVaultDraft } from '@/stores/create-vault-draft'

const OPTIONS = [
  {
    value: 'remote',
    title: 'Create a new private repository',
    hint: (owner: string, repo: string) => `${owner}/${repo} will be created for you`,
  },
  {
    value: 'local',
    title: 'Keep it on this computer only',
    hint: () => 'You can connect a GitHub repository later',
  },
] as const

export function RepositoryStep() {
  const navigate = useNavigate()
  const { data: account } = useAccount()
  const draft = useCreateVaultDraft()
  const [mode, setMode] = useState<'remote' | 'local'>(draft.createRemote ? 'remote' : 'local')
  const [repoName, setRepoName] = useState(draft.repoName || defaultRepoName(draft.name))

  const submit = (event: FormEvent) => {
    event.preventDefault()
    draft.update({ createRemote: mode === 'remote', repoName: repoName.trim() })
    void navigate({ to: '/vaults/new/$step', params: { step: 'password' } })
  }

  return (
    <form onSubmit={submit} className="flex flex-col gap-6">
      <PageHeading
        title="GitHub repository"
        description="Pick where the encrypted vault is stored."
      />
      <RadioGroup
        value={mode}
        onValueChange={(v) => setMode(v as 'remote' | 'local')}
        className="gap-3"
      >
        {OPTIONS.map((option) => (
          <label
            key={option.value}
            htmlFor={`repo-${option.value}`}
            className={cn(
              'flex cursor-pointer items-center gap-3 rounded-lg border p-3.5',
              mode === option.value ? 'border-info bg-info/10' : 'border-border bg-card',
            )}
          >
            <RadioGroupItem id={`repo-${option.value}`} value={option.value} />
            <span className="flex flex-col">
              <span className="text-sm font-semibold">{option.title}</span>
              <span className="text-xs text-muted-foreground">
                {option.hint(account?.login ?? 'you', repoName || 'vault')}
              </span>
            </span>
          </label>
        ))}
      </RadioGroup>
      {mode === 'remote' ? (
        <>
          <FormField label="Repository name" htmlFor="repo-name">
            <Input
              id="repo-name"
              value={repoName}
              onChange={(e) => setRepoName(e.target.value)}
              className="h-10 bg-inset font-mono"
            />
          </FormField>
          <Callout tone="info" icon={<Lock className="size-4 text-info" />}>
            Always private. Only encrypted data is pushed to GitHub.
          </Callout>
        </>
      ) : null}
      <WizardActions
        onBack={() => void navigate({ to: '/vaults/new/$step', params: { step: 'name' } })}
      />
    </form>
  )
}
