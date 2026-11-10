import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { WizardActions } from '@/components/general/wizard-actions'
import { Input } from '@/components/ui/input'
import { useFacets } from '@/hooks/use-secrets'
import { useAddSecretDraft } from '@/stores/add-secret-draft'

import { useAddSecretNav } from '../use-add-secret-nav'

const DEFAULT_ENVIRONMENTS = ['Production', 'Staging', 'Development']

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
  const { data: facets } = useFacets(vaultId)
  const { register, handleSubmit, formState } = useForm<Values>({
    resolver: zodResolver(schema),
    defaultValues: {
      name: draft.name,
      environment: draft.environment,
      tags: draft.tags,
    },
  })
  const environments = [
    ...new Set([...DEFAULT_ENVIRONMENTS, ...(facets?.environments.map((e) => e.name) ?? [])]),
  ]

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
        <Input
          id="secret-env"
          list="environments"
          className="h-10 bg-inset"
          {...register('environment')}
        />
        <datalist id="environments">
          {environments.map((e) => (
            <option key={e} value={e} />
          ))}
        </datalist>
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
