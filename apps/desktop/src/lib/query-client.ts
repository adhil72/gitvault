import { QueryClient } from '@tanstack/react-query'

const ONE_MINUTE_MS = 60_000

/** Single shared Query client. Secret values must never be stored in its cache. */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: { staleTime: ONE_MINUTE_MS, retry: 1, refetchOnWindowFocus: false },
  },
})
