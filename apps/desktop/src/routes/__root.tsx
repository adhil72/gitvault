import type { QueryClient } from '@tanstack/react-query'
import { createRootRouteWithContext, Outlet } from '@tanstack/react-router'

import { ThemeSync } from '@/components/general/theme-sync'
import { RootFrame } from '@/components/general/layouts/root-frame'
import { PagePlaceholder } from '@/components/general/page-placeholder'
import { Toaster } from '@/components/ui/sonner'

interface RouterContext {
  queryClient: QueryClient
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: () => (
    <RootFrame>
      <ThemeSync />
      <Outlet />
      <Toaster position="bottom-right" />
    </RootFrame>
  ),
  notFoundComponent: () => <PagePlaceholder title="Page not found" />,
})
