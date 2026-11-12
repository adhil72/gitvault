import type { VaultSummary } from '@/types'

/**
 * The vault to open right after switching GitHub accounts: the last opened one if it belongs
 * to that account, otherwise the account's first connected vault. `null` when it has none.
 */
export function pickVaultForAccount(vaults: VaultSummary[], login: string): VaultSummary | null {
  const owned = vaults.filter(
    (vault) => vault.id && vault.repo?.split('/')[0]?.toLowerCase() === login.toLowerCase(),
  )
  return owned.find((vault) => vault.current) ?? owned[0] ?? null
}
