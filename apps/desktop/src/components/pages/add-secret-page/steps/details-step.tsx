import { zodResolver } from '@hookform/resolvers/zod'
import { Controller, useForm } from 'react-hook-form'
import { z } from 'zod'

import { EnvironmentField } from '@/components/general/environment-field'
import { FormField } from '@/components/general/form-field'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { FIELD } from '../step-field-style'
import { StepActions } from '../step-actions'
import { StepFrame } from '../step-frame'
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
  const { vaultId, go, backToVault } = useAddSecretNav()
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
    <StepFrame
      title="Secret details"
      description="Name it the way your code reads it, then say where it is used."
      onSubmit={handleSubmit(onSubmit)}
      actions={
        <StepActions
          backLabel="Cancel"
          onBack={() => {
            draft.reset()
            backToVault()
          }}
        />
      }
    >
      <FormField
        label="Name"
        htmlFor="secret-name"
        error={formState.errors.name?.message}
        hint="Letters, digits, _ - and . are allowed."
      >
        <Input
          id="secret-name"
          autoFocus
          placeholder="STRIPE_SECRET_KEY"
          className={cn(FIELD, 'h-14 font-mono text-lg md:text-lg')}
          {...register('name')}
        />
      </FormField>
      <div className="grid gap-5 sm:grid-cols-2">
        <FormField label="Environment" htmlFor="secret-env" hint="e.g. Production, Staging">
          <Controller
            control={control}
            name="environment"
            render={({ field }) => (
              <EnvironmentField
                id="secret-env"
                value={field.value}
                onChange={field.onChange}
                vaultId={vaultId}
                className={FIELD}
              />
            )}
          />
        </FormField>
        <FormField label="Tags" htmlFor="secret-tags" hint="Separate with commas">
          <Input
            id="secret-tags"
            placeholder="billing, payments"
            className={FIELD}
            {...register('tags')}
          />
        </FormField>
      </div>
    </StepFrame>
  )
}
