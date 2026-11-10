import { createFileRoute } from '@tanstack/react-router'

import { VaultPage } from '@/components/pages/vault-page'
import { vaultSearchSchema } from '@/schemas/vault-search'

export const Route = createFileRoute('/_authed/vaults/$vaultId/')({
  validateSearch: vaultSearchSchema,
  component: VaultPage,
})
