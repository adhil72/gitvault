import { getWizardPanel, isStep, CREATE_VAULT_STEPS } from './wizards'

describe('wizard config', () => {
  it('picks the panel text from the URL', () => {
    expect(getWizardPanel('/vaults/new/password').headline).toBe('Protect your vault')
    expect(getWizardPanel('/vaults/abc/secrets/new/review').headline).toBe('Review and save')
    expect(getWizardPanel('/vaults/abc/unlock').headline).toBe('Welcome back')
  })

  it('validates step names', () => {
    expect(isStep(CREATE_VAULT_STEPS, 'recovery')).toBe(true)
    expect(isStep(CREATE_VAULT_STEPS, 'nope')).toBe(false)
  })
})
