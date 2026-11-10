import { defaultRepoName, slugify } from './slug'

describe('slug', () => {
  it('makes GitHub-safe names', () => {
    expect(slugify('  My Team Secrets! ')).toBe('my-team-secrets')
    expect(defaultRepoName('Personal vault')).toBe('gitvault-personal-vault')
  })
})
