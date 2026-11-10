import { createFileRoute } from '@tanstack/react-router'

import { AppShell } from '@/components/general/layouts/app-shell'
import { vaultGuard } from '@/lib/vault-guard'

// Everything inside an unlocked vault: header + sidebar + page.
export const Route = createFileRoute('/_authed/vaults/$vaultId')({
  beforeLoad: ({ context, params }) => vaultGuard(context.queryClient, params.vaultId),
  component: AppShell,
})
