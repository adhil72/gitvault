import { Link } from '@tanstack/react-router'
import { BookOpen } from 'lucide-react'
import { useState } from 'react'

import { CodeSnippet } from '@/components/general/code-snippet'
import { FrameworkSwitcher } from '@/components/general/framework-switcher'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { CONNECT_TARGETS, type ConnectStep, type ConnectTarget } from '@/config/connect-targets'

interface ConnectDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  vaultId: string
}

function StepItem({ step, index, last }: { step: ConnectStep; index: number; last: boolean }) {
  return (
    <li className="relative flex gap-3">
      {last ? null : (
        <span aria-hidden className="absolute top-7 bottom-[-1.25rem] left-3 w-px bg-border" />
      )}
      <span className="z-10 flex size-6 shrink-0 items-center justify-center rounded-full bg-primary/15 text-xs font-semibold text-primary">
        {index + 1}
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-2 pb-1">
        <div>
          <p className="font-medium">{step.title}</p>
          {step.hint ? <p className="text-xs text-muted-foreground">{step.hint}</p> : null}
        </div>
        <CodeSnippet {...step} copyLabel={`Copy step ${index + 1}`} />
      </div>
    </li>
  )
}

function TargetSteps({ target }: { target: ConnectTarget }) {
  return (
    <>
      <p className="text-muted-foreground">{target.description}</p>
      <ol className="flex flex-col gap-5">
        {target.steps.map((step, index) => (
          <StepItem
            key={step.title}
            step={step}
            index={index}
            last={index === target.steps.length - 1}
          />
        ))}
      </ol>
    </>
  )
}

/** Shows how to connect an app to the vault, one framework at a time from `CONNECT_TARGETS`. */
export function ConnectDialog({ open, onOpenChange, vaultId }: ConnectDialogProps) {
  const [framework, setFramework] = useState(CONNECT_TARGETS[0]?.id ?? '')
  const target = CONNECT_TARGETS.find((t) => t.id === framework)
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Connect an app</DialogTitle>
          <DialogDescription>
            Use this vault's secrets from your code. Values are decrypted on this device only.
          </DialogDescription>
        </DialogHeader>
        <FrameworkSwitcher value={framework} onValueChange={setFramework} />
        {target ? (
          <div className="flex flex-col gap-5 text-sm">
            <TargetSteps target={target} />
          </div>
        ) : null}
        <div className="flex items-center justify-between gap-3 border-t pt-4">
          <p className="text-xs text-muted-foreground">
            Servers, CI, options and troubleshooting are covered in the full guide.
          </p>
          <Button variant="secondary" asChild onClick={() => onOpenChange(false)}>
            <Link
              to="/vaults/$vaultId/docs/integration"
              params={{ vaultId }}
              search={{ framework }}
            >
              <BookOpen /> Detailed docs
            </Link>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
