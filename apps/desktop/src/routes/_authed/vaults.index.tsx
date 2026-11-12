import { createFileRoute, redirect } from '@tanstack/react-router'

import { resolveStartRoute } from '@/lib/resolve-start-route'

// No vault picker: /vaults sends you to your last used vault, or to the empty page.
export const Route = createFileRoute('/_authed/vaults/')({
  beforeLoad: async ({ context }) => {
    throw redirect(await resolveStartRoute(context.queryClient))
  },
})
