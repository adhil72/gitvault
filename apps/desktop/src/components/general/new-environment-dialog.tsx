import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, type FormEvent } from 'react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { FormField } from '@/components/general/form-field'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'

interface NewEnvironmentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Names that already exist, compared case-insensitively. */
  existing: readonly string[]
  initialName?: string
  onCreate: (name: string) => void
}

const makeSchema = (existing: readonly string[]) =>
  z.object({
    name: z
      .string()
      .trim()
      .min(1, 'Give the environment a name')
      .max(32, 'Keep it under 32 characters')
      .refine((name) => !existing.some((e) => e.toLowerCase() === name.toLowerCase()), {
        message: 'That environment already exists',
      }),
  })

/** Small dialog to name a new environment without leaving the form. */
export function NewEnvironmentDialog({
  open,
  onOpenChange,
  existing,
  initialName = '',
  onCreate,
}: NewEnvironmentDialogProps) {
  const { register, handleSubmit, reset, formState } = useForm<{ name: string }>({
    resolver: zodResolver(makeSchema(existing)),
    defaultValues: { name: initialName },
  })

  // Start from whatever the user had typed in the search box each time it opens.
  useEffect(() => {
    if (open) reset({ name: initialName })
  }, [open, initialName, reset])

  const submit = handleSubmit(({ name }) => {
    onCreate(name.trim())
    onOpenChange(false)
  })

  // The dialog is portaled, but React still bubbles its submit event to the
  // surrounding <form>; stop it so creating an environment never submits that form.
  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.stopPropagation()
    void submit(event)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <form onSubmit={onSubmit} className="flex flex-col gap-6">
          <DialogHeader>
            <DialogTitle>New environment</DialogTitle>
            <DialogDescription>
              Environments group secrets by where they are used, like Production or Staging.
            </DialogDescription>
          </DialogHeader>
          <FormField label="Name" htmlFor="new-env-name" error={formState.errors.name?.message}>
            <Input
              id="new-env-name"
              autoFocus
              autoComplete="off"
              placeholder="QA"
              className="h-10 bg-inset"
              {...register('name')}
            />
          </FormField>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit">Create environment</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
