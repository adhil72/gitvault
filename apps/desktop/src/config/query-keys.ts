import type { AuditQuery, ListSecretsQuery } from '@/types'

/** All TanStack Query keys in one place. Mutations invalidate by these prefixes. */
export const queryKeys = {
  auth: ['auth'] as const,
  ghAccounts: ['auth', 'gh-accounts'] as const,
  vaults: ['vaults'] as const,
  remoteVaults: ['vaults', 'remote'] as const,
  remoteStatus: (vaultId: string) => ['vault', vaultId, 'remote-status'] as const,
  vault: (vaultId: string) => ['vault', vaultId] as const,
  secrets: (vaultId: string) => ['secrets', vaultId] as const,
  secretList: (query: ListSecretsQuery) => ['secrets', query.vault_id, 'list', query] as const,
  facets: (vaultId: string) => ['secrets', vaultId, 'facets'] as const,
  secret: (vaultId: string, secretId: string) => ['secrets', vaultId, 'detail', secretId] as const,
  history: (vaultId: string, secretId: string) =>
    ['secrets', vaultId, 'history', secretId] as const,
  sync: (vaultId: string) => ['sync', vaultId] as const,
  conflicts: (vaultId: string) => ['sync', vaultId, 'conflicts'] as const,
  settings: ['settings'] as const,
  audit: (query: AuditQuery) => ['audit', query] as const,
}
