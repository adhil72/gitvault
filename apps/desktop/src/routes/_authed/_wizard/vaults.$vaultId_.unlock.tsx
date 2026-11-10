import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

import { UnlockPage } from '@/components/pages/unlock-page'

export const Route = createFileRoute('/_authed/_wizard/vaults/$vaultId_/unlock')({
  validateSearch: z.object({ redirect: z.string().optional() }),
  component: UnlockPage,
})
