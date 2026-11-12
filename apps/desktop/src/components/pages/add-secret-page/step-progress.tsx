import { ADD_SECRET_LABELS, ADD_SECRET_STEPS, type AddSecretStep } from '@/config/wizards'

const VISIBLE_STEPS = ADD_SECRET_STEPS.slice(0, -1)

/** A single hairline that fills as the wizard advances, with the step name above it. */
export function StepProgress({ current }: { current: AddSecretStep }) {
  const index = Math.max(VISIBLE_STEPS.indexOf(current), 0)
  const total = VISIBLE_STEPS.length
  return (
    <div
      role="progressbar"
      aria-label="Add secret progress"
      aria-valuemin={1}
      aria-valuemax={total}
      aria-valuenow={index + 1}
      aria-valuetext={`Step ${index + 1} of ${total}: ${ADD_SECRET_LABELS[current]}`}
      className="flex flex-col gap-3"
    >
      <div className="flex items-baseline justify-between text-[13px]">
        <span className="font-medium">{ADD_SECRET_LABELS[current]}</span>
        <span className="text-muted-foreground tabular-nums">
          {index + 1} / {total}
        </span>
      </div>
      <div className="h-0.5 overflow-hidden rounded-full bg-border">
        {/* scaleX keeps the fill on the compositor and lets it glide between steps. */}
        <div
          className="h-full origin-left rounded-full bg-foreground transition-transform duration-500 ease-settle motion-reduce:transition-none"
          style={{ transform: `scaleX(${(index + 1) / total})` }}
        />
      </div>
    </div>
  )
}
