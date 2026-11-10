export type StrengthLevel = 0 | 1 | 2 | 3 | 4

export interface Strength {
  level: StrengthLevel
  label: string
}

const LABELS = ['Too short', 'Weak', 'Fair', 'Strong', 'Very strong'] as const
const MIN_LENGTH = 8

function characterClasses(password: string): number {
  const tests = [/[a-z]/, /[A-Z]/, /[0-9]/, /[^a-zA-Z0-9]/]
  return tests.filter((test) => test.test(password)).length
}

/** A quick UX hint only: length and variety. The backend enforces the real minimum. */
export function passwordStrength(password: string): Strength {
  if (password.length < MIN_LENGTH) return { level: 0, label: LABELS[0] }
  const score =
    Math.min(Math.floor(password.length / 6), 3) + (characterClasses(password) >= 3 ? 1 : 0)
  const level = Math.min(Math.max(score, 1), 4) as StrengthLevel
  return { level, label: LABELS[level] }
}
