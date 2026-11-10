import { describeExpiry, fromDateInput, timeAgo, toDateInput } from './format'

describe('format helpers', () => {
  it('describes elapsed time', () => {
    const now = 1_000_000 * 1000
    expect(timeAgo(1_000_000 - 30, now)).toBe('just now')
    expect(timeAgo(1_000_000 - 3 * 86_400, now)).toBe('3 days ago')
    expect(timeAgo(1_000_000 - 3600, now)).toBe('1 hour ago')
  })

  it('round-trips date input values', () => {
    expect(toDateInput(fromDateInput('2026-12-04'))).toBe('2026-12-04')
    expect(fromDateInput('')).toBeNull()
    expect(toDateInput(null)).toBe('')
  })

  it('describes expiry in plain language', () => {
    expect(describeExpiry(null)).toBeNull()
    expect(describeExpiry(12)).toBe('Expires in 12 days')
    expect(describeExpiry(-1)).toBe('Expired 1 day ago')
    expect(describeExpiry(0)).toBe('Expires today')
  })
})
