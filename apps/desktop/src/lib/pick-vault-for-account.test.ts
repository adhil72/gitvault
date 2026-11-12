import { describe, expect, it } from 'vitest'

import type { VaultSummary } from '@/types'

import { pickVaultForAccount } from './pick-vault-for-account'

const vault = (name: string, repo: string | null, current = false) =>
  ({ id: name, name, repo, current }) as unknown as VaultSummary

describe('pickVaultForAccount', () => {
  const vaults = [vault('a', 'bob/a'), vault('b', 'ann/b'), vault('c', 'ann/c', true)]

  it('prefers the last opened vault of the account', () => {
    expect(pickVaultForAccount(vaults, 'ann')?.name).toBe('c')
  })

  it('falls back to the first vault of the account', () => {
    expect(pickVaultForAccount(vaults, 'bob')?.name).toBe('a')
  })

  it('returns null when the account has no vault', () => {
    expect(pickVaultForAccount(vaults, 'zed')).toBeNull()
  })
})
