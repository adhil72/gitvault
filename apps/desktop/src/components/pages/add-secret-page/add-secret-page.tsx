import { useParams } from '@tanstack/react-router'

import { ADD_SECRET_STEPS, isStep } from '@/config/wizards'

import { StepProgress } from './step-progress'
import { DetailsStep } from './steps/details-step'
import { ExpiryStep } from './steps/expiry-step'
import { ReviewStep } from './steps/review-step'
import { SavedStep } from './steps/saved-step'
import { ValueStep } from './steps/value-step'

const STEP_COMPONENTS = {
  details: DetailsStep,
  value: ValueStep,
  expiry: ExpiryStep,
  review: ReviewStep,
  saved: SavedStep,
} as const

/** The add-secret wizard: one calm column per route step, no surrounding card. */
export function AddSecretPage() {
  const { step } = useParams({ from: '/_authed/_wizard/vaults/$vaultId_/secrets/new/$step' })
  if (!isStep(ADD_SECRET_STEPS, step)) return null
  const Step = STEP_COMPONENTS[step]
  return (
    <div className="flex w-full max-w-[460px] flex-col gap-12">
      {step === 'saved' ? null : <StepProgress current={step} />}
      <Step />
    </div>
  )
}
