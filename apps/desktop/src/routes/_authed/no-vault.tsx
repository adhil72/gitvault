import { createFileRoute } from '@tanstack/react-router'

import { NoVaultPage } from '@/components/pages/no-vault-page'

export const Route = createFileRoute('/_authed/no-vault')({
  component: NoVaultPage,
})
