import { Button } from '@/components/ui/button'

interface WizardActionsProps {
  onBack: () => void
  backLabel?: string
  nextLabel?: string
  nextType?: 'button' | 'submit'
  onNext?: () => void
  nextDisabled?: boolean
  busy?: boolean
}

/** Back (ghost) and Continue (primary) buttons used at the bottom of every wizard step. */
export function WizardActions({
  onBack,
  backLabel = 'Back',
  nextLabel = 'Continue',
  nextType = 'submit',
  onNext,
  nextDisabled,
  busy,
}: WizardActionsProps) {
  return (
    <div className="flex gap-3">
      <Button type="button" variant="ghost" onClick={onBack}>
        {backLabel}
      </Button>
      <Button
        type={nextType}
        className="flex-1"
        disabled={nextDisabled || busy}
        {...(onNext ? { onClick: onNext } : {})}
      >
        {busy ? 'Working…' : nextLabel}
      </Button>
    </div>
  )
}
