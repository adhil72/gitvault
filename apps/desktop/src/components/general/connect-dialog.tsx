import { CopyButton } from '@/components/general/copy-button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { CONNECT_TARGETS, type ConnectStep, type ConnectTarget } from '@/config/connect-targets'

interface ConnectDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

function CodeStep({ step, index }: { step: ConnectStep; index: number }) {
  return (
    <li className="flex flex-col gap-1.5">
      <p className="font-medium">
        {index + 1}. {step.title}
      </p>
      {step.hint ? <p className="text-xs text-muted-foreground">{step.hint}</p> : null}
      <div className="flex items-start justify-between gap-2 rounded-lg bg-muted/60 py-2 pr-1 pl-3">
        <pre className="overflow-x-auto font-mono text-[13px] leading-relaxed">
          <code>{step.code}</code>
        </pre>
        <CopyButton value={step.code} label={`Copy step ${index + 1}`} />
      </div>
    </li>
  )
}

function TargetSteps({ target }: { target: ConnectTarget }) {
  return (
    <>
      <p className="text-muted-foreground">{target.description}</p>
      <ol className="flex flex-col gap-4">
        {target.steps.map((step, index) => (
          <CodeStep key={step.title} step={step} index={index} />
        ))}
      </ol>
    </>
  )
}

/** Shows how to connect an app to the vault, one tab per runtime in `CONNECT_TARGETS`. */
export function ConnectDialog({ open, onOpenChange }: ConnectDialogProps) {
  const first = CONNECT_TARGETS[0]
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>Connect an app</DialogTitle>
          <DialogDescription>
            Use this vault's secrets from your code. Values are decrypted on this device only.
          </DialogDescription>
        </DialogHeader>
        {first ? (
          <Tabs defaultValue={first.id} className="gap-4">
            {CONNECT_TARGETS.length > 1 ? (
              <TabsList>
                {CONNECT_TARGETS.map((target) => (
                  <TabsTrigger key={target.id} value={target.id}>
                    {target.label}
                  </TabsTrigger>
                ))}
              </TabsList>
            ) : null}
            {CONNECT_TARGETS.map((target) => (
              <TabsContent key={target.id} value={target.id} className="flex flex-col gap-4">
                <TargetSteps target={target} />
              </TabsContent>
            ))}
          </Tabs>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
