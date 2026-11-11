import { createFileRoute } from '@tanstack/react-router'
import { VaultsHomePage } from '@/components/pages/vaults-home-page'

export const Route = createFileRoute('/_authed/vaults/')({
  component: VaultsHomePage,
})
