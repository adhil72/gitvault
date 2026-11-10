import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect } from 'react'
import { Controller, useForm } from 'react-hook-form'
import { z } from 'zod'

import { DatePicker } from '@/components/general/date-picker'
import { ErrorNotice } from '@/components/general/error-notice'
import { FormField } from '@/components/general/form-field'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Textarea } from '@/components/ui/textarea'
import { useSecret, useUpdateSecret } from '@/hooks/use-secrets'
import { fromDateInput, toDateInput } from '@/lib/format'

import type { useVaultNav } from './use-vault-nav'

const schema = z.object({
  name: z
    .string()
    .trim()
    .min(1, 'Give the secret a name')
    .regex(/^[A-Za-z0-9_.-]+$/, 'Use letters, digits, _ - and . only'),
  environment: z.string(),
  tags: z.string(),
  value: z.string(),
  expires: z.string(),
  notes: z.string(),
})

type Values = z.infer<typeof schema>

/** Side sheet to edit a secret. Leave the value empty to keep it. */
export function EditSecretSheet({ nav }: { nav: ReturnType<typeof useVaultNav> }) {
  const open = nav.search.panel === 'edit' && Boolean(nav.search.secret)
  const { data: secret } = useSecret(nav.vaultId, open ? nav.search.secret : undefined)
  const update = useUpdateSecret(nav.vaultId)
  const { register, handleSubmit, reset, control, formState } = useForm<Values>({
    resolver: zodResolver(schema),
  })

  useEffect(() => {
    if (!secret) return
    reset({
      name: secret.name,
      environment: secret.environment ?? '',
      tags: secret.tags.join(', '),
      value: '',
      expires: toDateInput(secret.expires_at),
      notes: secret.notes,
    })
  }, [secret, reset])

  const onSubmit = (values: Values) => {
    if (!secret) return
    update.mutate(
      {
        vault_id: nav.vaultId,
        secret_id: secret.id,
        name: values.name.trim(),
        kind: secret.kind,
        value: values.value || null,
        environment: values.environment || null,
        tags: values.tags
          .split(',')
          .map((t) => t.trim())
          .filter(Boolean),
        notes: values.notes || null,
        expires_at: fromDateInput(values.expires),
        remind_before_days: secret.remind_before_days,
        favorite: secret.favorite,
      },
      { onSuccess: () => nav.closePanel() },
    )
  }

  return (
    <Sheet open={open} onOpenChange={(next) => !next && nav.closePanel()}>
      <SheetContent className="w-[420px] gap-0 sm:max-w-[420px]">
        <SheetHeader>
          <SheetTitle>Edit secret</SheetTitle>
          <SheetDescription>Saving creates a new version and pushes a commit.</SheetDescription>
        </SheetHeader>
        <form
          onSubmit={handleSubmit(onSubmit)}
          className="flex flex-1 flex-col gap-4 overflow-y-auto px-4 pb-4"
        >
          <FormField label="Name" htmlFor="edit-name" error={formState.errors.name?.message}>
            <Input id="edit-name" className="h-10 bg-inset font-mono" {...register('name')} />
          </FormField>
          <FormField
            label="New value"
            htmlFor="edit-value"
            hint="Leave empty to keep the current value"
          >
            <Textarea
              id="edit-value"
              autoComplete="off"
              spellCheck={false}
              className="min-h-16 bg-inset font-mono text-[13px]"
              {...register('value')}
            />
          </FormField>
          <div className="grid grid-cols-2 gap-3">
            <FormField label="Environment" htmlFor="edit-env">
              <Input id="edit-env" className="h-10 bg-inset" {...register('environment')} />
            </FormField>
          </div>
          <FormField label="Tags" htmlFor="edit-tags" hint="Separate with commas">
            <Input id="edit-tags" className="h-10 bg-inset" {...register('tags')} />
          </FormField>
          <FormField label="Expires" htmlFor="edit-expires">
            <Controller
              control={control}
              name="expires"
              render={({ field }) => (
                <DatePicker id="edit-expires" value={field.value ?? ''} onChange={field.onChange} />
              )}
            />
          </FormField>
          <FormField label="Notes" htmlFor="edit-notes">
            <Textarea id="edit-notes" className="min-h-16 bg-inset" {...register('notes')} />
          </FormField>
          <ErrorNotice error={update.error} />
          <SheetFooter className="mt-auto flex-row justify-end gap-3 p-0">
            <Button type="button" variant="ghost" onClick={nav.closePanel}>
              Cancel
            </Button>
            <Button type="submit" disabled={update.isPending}>
              {update.isPending ? 'Saving…' : 'Save changes'}
            </Button>
          </SheetFooter>
        </form>
      </SheetContent>
    </Sheet>
  )
}
