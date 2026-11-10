import { cn } from '@/lib/utils'

interface WizardStepsProps {
  steps: readonly string[]
  labels: Record<string, string>
  current: string
}

/** A progress bar with one segment per step and a "Step N of M" caption. */
export function WizardSteps({ steps, labels, current }: WizardStepsProps) {
  const index = Math.max(steps.indexOf(current), 0)
  return (
    <div className="flex flex-col gap-2" aria-label="Progress">
      <div className="flex gap-1.5">
        {steps.map((step, i) => (
          <span
            key={step}
            className={cn('h-1 flex-1 rounded-full', i <= index ? 'bg-info' : 'bg-muted')}
          />
        ))}
      </div>
      <p className="text-xs font-medium text-muted-foreground">
        Step {index + 1} of {steps.length} · {labels[current] ?? current}
      </p>
    </div>
  )
}
