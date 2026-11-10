import { createHashHistory, createRouter } from '@tanstack/react-router'

import { routeTree } from '@/routeTree.gen'

import { queryClient } from './query-client'

/** Hash history avoids 404s on reload under Tauri's custom protocol. */
export const router = createRouter({
  routeTree,
  history: createHashHistory(),
  context: { queryClient },
  defaultPreload: 'intent',
  scrollRestoration: true,
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
