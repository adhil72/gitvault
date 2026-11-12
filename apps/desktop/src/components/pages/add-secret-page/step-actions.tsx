import { ArrowRight } from 'lucide-react'
import type { ReactNode } from 'react'

import { Button } from '@/components/ui/button'

interface StepActionsProps {
  onBack: () => void
  backLabel?: string
  nextLabel?: string
  nextIcon?: ReactNode
  nextType?: 'button' | 'submit'
  onNext?: () => void
  nextDisabled?: boolean
  busy?: boolean
}

/** A quiet Back on the left, the primary action on the right. */
export function StepActions({
  onBack,
  backLabel = 'Back',
  nextLabel = 'Continue',
  nextIcon = <ArrowRight data-icon="inline-end" />,
  nextType = 'submit',
  onNext,
  nextDisabled,
  busy,
}: StepActionsProps) {
  return (
    <>
      <Button
        type="button"
        variant="ghost"
        className="h-12 rounded-xl px-4 text-[15px] text-muted-foreground hover:text-foreground"
        onClick={onBack}
      >
        {backLabel}
      </Button>
      <Button
        type={nextType}
        className="h-12 min-w-40 rounded-xl px-7 text-[15px] font-semibold"
        disabled={nextDisabled || busy}
        {...(onNext ? { onClick: onNext } : {})}
      >
        {busy ? 'Saving…' : nextLabel}
        {busy ? null : nextIcon}
      </Button>
    </>
  )
}
