import { useParams } from '@tanstack/react-router'

import { CREATE_VAULT_LABELS, CREATE_VAULT_STEPS, isStep } from '@/config/wizards'
import { WizardSteps } from '@/components/general/wizard-steps'

import { DoneStep } from './steps/done-step'
import { NameStep } from './steps/name-step'
import { PasswordStep } from './steps/password-step'
import { RecoveryStep } from './steps/recovery-step'
import { RepositoryStep } from './steps/repository-step'

const STEP_COMPONENTS = {
  name: NameStep,
  repository: RepositoryStep,
  password: PasswordStep,
  recovery: RecoveryStep,
  done: DoneStep,
} as const

/** The five pages of the create-vault wizard, one per route step. */
export function CreateVaultPage() {
  const { step } = useParams({ from: '/_authed/_wizard/vaults/new/$step' })
  if (!isStep(CREATE_VAULT_STEPS, step)) return null
  const Step = STEP_COMPONENTS[step]
  return (
    <div className="flex w-[360px] flex-col gap-6">
      {step === 'done' ? null : (
        <WizardSteps
          steps={CREATE_VAULT_STEPS.slice(0, 4)}
          labels={CREATE_VAULT_LABELS}
          current={step}
        />
      )}
      <Step />
    </div>
  )
}
