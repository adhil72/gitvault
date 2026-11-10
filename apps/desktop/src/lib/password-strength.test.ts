import { passwordStrength } from './password-strength'

describe('passwordStrength', () => {
  it('treats short passwords as too short', () => {
    expect(passwordStrength('abc')).toEqual({ level: 0, label: 'Too short' })
  })

  it('rewards length and character variety', () => {
    expect(passwordStrength('aaaaaaaa').level).toBeLessThan(
      passwordStrength('Tr0ub4dor&3-horse-battery').level,
    )
    expect(passwordStrength('Tr0ub4dor&3-horse-battery').level).toBe(4)
  })
})
