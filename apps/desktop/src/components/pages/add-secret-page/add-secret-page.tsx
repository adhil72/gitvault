import { useParams } from '@tanstack/react-router'

import { WizardSteps } from '@/components/general/wizard-steps'
import { ADD_SECRET_LABELS, ADD_SECRET_STEPS, isStep } from '@/config/wizards'

import { DetailsStep } from './steps/details-step'
import { ExpiryStep } from './steps/expiry-step'
import { ReviewStep } from './steps/review-step'
import { SavedStep } from './steps/saved-step'
import { TypeStep } from './steps/type-step'
import { ValueStep } from './steps/value-step'

const STEP_COMPONENTS = {
  type: TypeStep,
  details: DetailsStep,
  value: ValueStep,
  expiry: ExpiryStep,
  review: ReviewStep,
  saved: SavedStep,
} as const

/** The six pages of the add-secret wizard, one per route step. */
export function AddSecretPage() {
  const { step } = useParams({ from: '/_authed/_wizard/vaults/$vaultId_/secrets/new/$step' })
  if (!isStep(ADD_SECRET_STEPS, step)) return null
  const Step = STEP_COMPONENTS[step]
  return (
    <div className="flex w-[360px] flex-col gap-6">
      {step === 'saved' ? null : (
        <WizardSteps
          steps={ADD_SECRET_STEPS.slice(0, 5)}
          labels={ADD_SECRET_LABELS}
          current={step}
        />
      )}
      <Step />
    </div>
  )
}
