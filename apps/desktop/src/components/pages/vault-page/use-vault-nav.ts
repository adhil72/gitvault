import { useNavigate, useParams, useSearch } from '@tanstack/react-router'

import type { VaultSearch } from '@/schemas/vault-search'

type Panel = NonNullable<VaultSearch['panel']>

/** Navigation helpers for the vault page: select a secret, open or close a panel, change sort. */
export function useVaultNav() {
  const navigate = useNavigate()
  const { vaultId } = useParams({ from: '/_authed/vaults/$vaultId/' })
  const search = useSearch({ from: '/_authed/vaults/$vaultId/' })
  const update = (next: VaultSearch) =>
    void navigate({ to: '/vaults/$vaultId', params: { vaultId }, search: next })
  return {
    vaultId,
    search,
    select: (secret: string | undefined) => update({ ...search, secret, panel: undefined }),
    openPanel: (panel: Panel) => update({ ...search, panel }),
    closePanel: () => update({ ...search, panel: undefined }),
    clearSelection: () => update({ ...search, secret: undefined, panel: undefined }),
    setSort: (sort: VaultSearch['sort']) => update({ ...search, sort }),
  }
}
