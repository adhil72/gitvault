import { cn } from './utils'

describe('cn', () => {
  const isHidden = Boolean(0)

  it('merges class names and resolves Tailwind conflicts', () => {
    expect(cn('px-2', 'px-4', isHidden && 'hidden')).toBe('px-4')
  })
})
