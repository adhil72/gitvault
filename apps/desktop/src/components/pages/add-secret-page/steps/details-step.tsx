import { zodResolver } from '@hookform/resolvers/zod'
import { Controller, useForm } from 'react-hook-form'
import { z } from 'zod'

import { EnvironmentField } from '@/components/general/environment-field'
import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { Input } from '@/components/ui/input'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { useAddSecretNav } from '../use-add-secret-nav'

const schema = z.object({
  name: z
    .string()
    .trim()
    .min(1, 'Give the secret a name')
    .regex(/^[A-Za-z0-9_.-]+$/, 'Use letters, digits, _ - and . only'),
  environment: z.string(),
  tags: z.string(),
})

type Values = z.infer<typeof schema>

export function DetailsStep() {
  const { vaultId, go } = useAddSecretNav()
  const draft = useAddSecretDraft()
  const { register, handleSubmit, control, formState } = useForm<Values>({
    resolver: zodResolver(schema),
    defaultValues: {
      name: draft.name,
      environment: draft.environment,
      tags: draft.tags,
    },
  })

  const onSubmit = (values: Values) => {
    draft.update({ ...values, name: values.name.trim() })
    go('value')
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-5">
      <PageHeading title="Secret details" description="Where does this secret belong?" />
      <FormField label="Name" htmlFor="secret-name" error={formState.errors.name?.message}>
        <Input
          id="secret-name"
          autoFocus
          placeholder="STRIPE_SECRET_KEY"
          className="h-10 bg-inset font-mono"
          {...register('name')}
        />
      </FormField>
      <FormField label="Environment" htmlFor="secret-env" hint="Where it is used, e.g. Production">
        <Controller
          control={control}
          name="environment"
          render={({ field }) => (
            <EnvironmentField
              id="secret-env"
              value={field.value}
              onChange={field.onChange}
              vaultId={vaultId}
            />
          )}
        />
      </FormField>
      <FormField label="Tags" htmlFor="secret-tags" hint="Separate with commas">
        <Input
          id="secret-tags"
          placeholder="billing, payments"
          className="h-10 bg-inset"
          {...register('tags')}
        />
      </FormField>
      <WizardActions onBack={() => go('type')} />
    </form>
  )
}
