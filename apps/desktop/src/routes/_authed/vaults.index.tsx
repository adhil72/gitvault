import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

import { VaultsHomePage } from '@/components/pages/vaults-home-page'

export const Route = createFileRoute('/_authed/vaults/')({
  // `?all=1` shows the vault list instead of jumping straight into the current vault.
  validateSearch: z.object({ all: z.boolean().optional() }),
  component: VaultsHomePage,
})
