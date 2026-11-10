import { createFileRoute } from '@tanstack/react-router'

import { GeneratorPage } from '@/components/pages/generator-page'

export const Route = createFileRoute('/_authed/vaults/$vaultId/tools/generator')({
  component: GeneratorPage,
})
