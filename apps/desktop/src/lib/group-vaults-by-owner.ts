import type { VaultSummary } from '@/types'

export interface VaultGroup {
  /** GitHub account/org the repositories belong to, or `null` for vaults with no remote. */
  owner: string | null
  vaults: VaultSummary[]
}

const ownerOf = (vault: VaultSummary): string | null => vault.repo?.split('/')[0] ?? null

/** Groups vaults by GitHub owner (alphabetical); local-only vaults go last in their own group. */
export function groupVaultsByOwner(vaults: VaultSummary[]): VaultGroup[] {
  const groups = new Map<string | null, VaultSummary[]>()
  for (const vault of vaults) {
    const owner = ownerOf(vault)
    groups.set(owner, [...(groups.get(owner) ?? []), vault])
  }
  return [...groups.entries()]
    .sort(([a], [b]) => (a === null ? 1 : b === null ? -1 : a.localeCompare(b)))
    .map(([owner, list]) => ({ owner, vaults: list }))
}
