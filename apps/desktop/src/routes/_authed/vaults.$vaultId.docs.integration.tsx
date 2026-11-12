import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

import { DocsPage } from '@/components/pages/docs-page'

export const Route = createFileRoute('/_authed/vaults/$vaultId/docs/integration')({
  validateSearch: z.object({ framework: z.string().optional() }),
  component: DocsPage,
})
