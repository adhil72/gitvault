import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
} from '@tanstack/react-router'
import { render } from '@testing-library/react'
import { ThemeProvider } from 'next-themes'
import type { FunctionComponent, ReactNode } from 'react'

import { resetMock } from '@/api/mock/mock-backend'
import { routeTree } from '@/routeTree.gen'

function newQueryClient() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } })
}

function Providers({ client, children }: { client: QueryClient; children: ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark">
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    </ThemeProvider>
  )
}

/** Renders a page component inside a minimal in-memory router (so <Link> works). */
export async function renderPage(Page: FunctionComponent) {
  const client = newQueryClient()
  const rootRoute = createRootRoute({ component: Page })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  const result = render(
    <Providers client={client}>
      <RouterProvider router={router} />
    </Providers>,
  )
  await router.load()
  return result
}

/** Renders the whole app at `path` against a fresh mock backend. Seed data first if needed. */
export async function renderApp(path: string) {
  const client = newQueryClient()
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [path] }),
    context: { queryClient: client },
  })
  const result = render(
    <Providers client={client}>
      <RouterProvider router={router} />
    </Providers>,
  )
  await router.load()
  return { ...result, router }
}

export { resetMock }
