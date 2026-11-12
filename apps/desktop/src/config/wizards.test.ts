import { isStep, CREATE_VAULT_STEPS } from './wizards'

describe('wizard config', () => {
  it('validates step names', () => {
    expect(isStep(CREATE_VAULT_STEPS, 'recovery')).toBe(true)
    expect(isStep(CREATE_VAULT_STEPS, 'nope')).toBe(false)
  })
})
