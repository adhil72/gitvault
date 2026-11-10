import { createFileRoute, Outlet, redirect } from '@tanstack/react-router'

import { authQueryOptions } from '@/hooks/use-auth'

// Guard: everything below requires a GitHub session.
export const Route = createFileRoute('/_authed')({
  beforeLoad: async ({ context }) => {
    const account = await context.queryClient.ensureQueryData(authQueryOptions())
    if (!account) throw redirect({ to: '/sign-in' })
  },
  component: Outlet,
})
