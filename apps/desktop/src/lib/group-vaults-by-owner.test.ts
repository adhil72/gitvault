import { describe, expect, it } from 'vitest'

import type { VaultSummary } from '@/types'

import { groupVaultsByOwner } from './group-vaults-by-owner'

const vault = (name: string, repo: string | null) =>
  ({
    id: name,
    name,
    description: '',
    repo,
    state: 'ready',
    unlocked: false,
    current: false,
  }) as unknown as VaultSummary

describe('groupVaultsByOwner', () => {
  it('groups by owner, sorted, with local-only last', () => {
    const groups = groupVaultsByOwner([
      vault('a', null),
      vault('b', 'zed/b'),
      vault('c', 'adhil72/c'),
      vault('d', 'zed/d'),
    ])
    expect(groups.map((g) => g.owner)).toEqual(['adhil72', 'zed', null])
    expect(groups[1]?.vaults.map((v) => v.name)).toEqual(['b', 'd'])
  })
})
