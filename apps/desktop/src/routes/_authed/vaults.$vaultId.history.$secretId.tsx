import { createFileRoute } from '@tanstack/react-router'

import { SecretHistoryPage } from '@/components/pages/secret-history-page'

export const Route = createFileRoute('/_authed/vaults/$vaultId/history/$secretId')({
  component: SecretHistoryPage,
})
