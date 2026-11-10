import { buildQuery, hasFilter, titleFor } from './build-query'

describe('vault page query', () => {
  it('maps search parameters to a backend query', () => {
    const query = buildQuery('v1', { environment: 'Production', view: 'expiring', sort: 'name' })
    expect(query).toMatchObject({
      vault_id: 'v1',
      environment: 'Production',
      expiring_within_days: 30,
      favorites_only: false,
      sort: 'name',
    })
  })

  it('titles the page after the active filter', () => {
    expect(titleFor({})).toBe('All secrets')
    expect(titleFor({ environment: 'production' })).toBe('production')
    expect(titleFor({ view: 'favorites' })).toBe('Favorites')
    expect(hasFilter({ q: 'x' })).toBe(true)
    expect(hasFilter({})).toBe(false)
  })
})
