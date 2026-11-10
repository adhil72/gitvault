import type { ReactNode } from 'react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

interface ConfirmDialogProps {
  open: boolean
  title: string
  description: string
  confirmLabel: string
  destructive?: boolean
  busy?: boolean
  confirmDisabled?: boolean
  children?: ReactNode
  onConfirm: () => void
  onCancel: () => void
}

/** A modal that asks the user to confirm an action. */
export function ConfirmDialog(props: ConfirmDialogProps) {
  const { open, title, description, confirmLabel, destructive, busy, confirmDisabled, children } =
    props
  return (
    <Dialog open={open} onOpenChange={(next) => !next && props.onCancel()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        {children}
        <DialogFooter>
          <Button variant="ghost" onClick={props.onCancel}>
            Cancel
          </Button>
          <Button
            variant={destructive ? 'danger' : 'default'}
            disabled={busy || confirmDisabled}
            onClick={props.onConfirm}
          >
            {busy ? 'Working…' : confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
