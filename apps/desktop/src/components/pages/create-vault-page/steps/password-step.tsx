import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate } from '@tanstack/react-router'
import { TriangleAlert } from 'lucide-react'
import { useForm, useWatch } from 'react-hook-form'
import { z } from 'zod'

import { Callout } from '@/components/general/callout'
import { ErrorNotice } from '@/components/general/error-notice'
import { FormField } from '@/components/general/form-field'
import { PageHeading } from '@/components/general/page-heading'
import { PasswordField } from '@/components/general/password-field'
import { StrengthMeter } from '@/components/general/strength-meter'
import { WizardActions } from '@/components/general/wizard-actions'
import { useCreateVault } from '@/hooks/use-vaults'
import { useCreateVaultDraft } from '@/stores/create-vault-draft'

const schema = z
  .object({ password: z.string().min(8, 'Use at least 8 characters'), confirm: z.string() })
  .refine((v) => v.password === v.confirm, {
    message: 'The passwords do not match',
    path: ['confirm'],
  })

type Values = z.infer<typeof schema>

export function PasswordStep() {
  const navigate = useNavigate()
  const draft = useCreateVaultDraft()
  const create = useCreateVault()
  const { register, handleSubmit, control, formState } = useForm<Values>({
    resolver: zodResolver(schema),
  })

  const password = useWatch({ control, name: 'password' })

  const onSubmit = (values: Values) => {
    create.mutate(
      {
        name: draft.name,
        description: draft.description || null,
        repo_name: draft.repoName || null,
        create_remote: draft.createRemote,
        master_password: values.password,
      },
      {
        onSuccess: (created) => {
          draft.update({
            result: {
              vaultId: created.vault.id ?? '',
              vaultName: created.vault.name,
              recoveryKey: created.recovery_key,
              repo: created.vault.repo,
              remoteError: created.remote_error,
            },
          })
          void navigate({ to: '/vaults/new/$step', params: { step: 'recovery' } })
        },
      },
    )
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-5">
      <PageHeading
        title="Set a master password"
        description="You will need it every time you unlock this vault."
      />
      <FormField label="Master password" htmlFor="mp" error={formState.errors.password?.message}>
        <PasswordField id="mp" autoFocus autoComplete="new-password" {...register('password')} />
      </FormField>
      <StrengthMeter password={password ?? ''} />
      <FormField
        label="Confirm master password"
        htmlFor="mp2"
        error={formState.errors.confirm?.message}
      >
        <PasswordField id="mp2" autoComplete="new-password" {...register('confirm')} />
      </FormField>
      <Callout tone="warning" icon={<TriangleAlert className="size-4 text-warning" />}>
        We cannot recover your master password. Keep your recovery key safe.
      </Callout>
      <ErrorNotice error={create.error} />
      <WizardActions
        busy={create.isPending}
        nextLabel="Create vault"
        onBack={() => void navigate({ to: '/vaults/new/$step', params: { step: 'repository' } })}
      />
    </form>
  )
}
