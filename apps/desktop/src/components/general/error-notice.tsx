import { TriangleAlert } from 'lucide-react'

import { Callout } from '@/components/general/callout'
import { ApiError } from '@/lib/api-error'

/** Shows a backend error (message plus the hint, when there is one). */
export function ErrorNotice({ error }: { error: unknown }) {
  if (!error) return null
  const message = error instanceof Error ? error.message : 'Something went wrong'
  const hint = error instanceof ApiError ? error.hint : null
  return (
    <Callout tone="warning" icon={<TriangleAlert className="size-4 text-warning" />}>
      <p>{message}</p>
      {hint ? <p className="mt-1 font-mono text-xs text-muted-foreground">{hint}</p> : null}
    </Callout>
  )
}
