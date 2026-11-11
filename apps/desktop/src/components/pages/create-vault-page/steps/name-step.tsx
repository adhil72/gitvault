import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate } from '@tanstack/react-router'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { Input } from '@/components/ui/input'
import { useCreateVaultDraft } from '@/stores/create-vault-draft'

import { AccountField } from './account-field'

const schema = z.object({
  name: z.string().trim().min(1, 'Give the vault a name').max(80, 'Use at most 80 characters'),
  description: z.string().max(200, 'Use at most 200 characters'),
})

type Values = z.infer<typeof schema>

export function NameStep() {
  const navigate = useNavigate()
  const draft = useCreateVaultDraft()
  const { register, handleSubmit, formState } = useForm<Values>({
    resolver: zodResolver(schema),
    defaultValues: { name: draft.name, description: draft.description },
  })

  const onSubmit = (values: Values) => {
    draft.update({ name: values.name.trim(), description: values.description })
    void navigate({ to: '/vaults/new/$step', params: { step: 'repository' } })
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-6">
      <PageHeading title="Name your vault" description="What should we call it?" />
      <FormField label="Vault name" htmlFor="vault-name" error={formState.errors.name?.message}>
        <Input
          id="vault-name"
          autoFocus
          placeholder="Personal vault"
          className="h-10 bg-inset"
          {...register('name')}
        />
      </FormField>
      <AccountField />
      <FormField
        label="Description (optional)"
        htmlFor="vault-description"
        error={formState.errors.description?.message}
      >
        <Input
          id="vault-description"
          placeholder="Shared API keys for the platform team"
          className="h-10 bg-inset"
          {...register('description')}
        />
      </FormField>
      <WizardActions
        backLabel="Cancel"
        onBack={() => void navigate({ to: '/vaults', search: { all: true } })}
      />
    </form>
  )
}
