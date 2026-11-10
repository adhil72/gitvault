import type { VaultSearch } from '@/schemas/vault-search'
import type { ListSecretsQuery } from '@/types'

const EXPIRING_SOON_DAYS = 30

/** Turns the URL search parameters into a backend query. */
export function buildQuery(vaultId: string, search: VaultSearch): ListSecretsQuery {
  return {
    vault_id: vaultId,
    search: search.q ?? null,
    environment: search.environment ?? null,
    tag: search.tag ?? null,
    kind: null,
    favorites_only: search.view === 'favorites',
    expiring_within_days: search.view === 'expiring' ? EXPIRING_SOON_DAYS : null,
    sort: search.sort ?? null,
  }
}

/** Page title for the active filter. */
export function titleFor(search: VaultSearch): string {
  if (search.environment) return search.environment
  if (search.tag) return `#${search.tag}`
  if (search.view === 'favorites') return 'Favorites'
  if (search.view === 'expiring') return 'Expiring soon'
  if (search.q) return `Search: ${search.q}`
  return 'All secrets'
}

/** True when any filter narrows the list. */
export function hasFilter(search: VaultSearch): boolean {
  return Boolean(search.q ?? search.environment ?? search.tag ?? search.view)
}
