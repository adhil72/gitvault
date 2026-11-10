import { createFileRoute, redirect } from '@tanstack/react-router'

import { AuthSplitLayout } from '@/components/general/layouts/auth-split-layout'
import { authQueryOptions } from '@/hooks/use-auth'

// Signed-in users skip the sign-in screens.
export const Route = createFileRoute('/_public')({
  beforeLoad: async ({ context }) => {
    const account = await context.queryClient.ensureQueryData(authQueryOptions())
    if (account) throw redirect({ to: '/vaults' })
  },
  component: AuthSplitLayout,
})
