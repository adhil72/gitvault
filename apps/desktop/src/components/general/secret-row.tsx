import { Copy, KeyRound, Star } from 'lucide-react'

import { EnvBadge } from '@/components/general/env-badge'
import { Button } from '@/components/ui/button'
import { describeExpiry, timeAgo } from '@/lib/format'
import { cn } from '@/lib/utils'
import type { SecretSummary } from '@/types'

interface SecretRowProps {
  secret: SecretSummary
  selected: boolean
  onSelect: () => void
  onCopy: () => void
}

const EXPIRY_WARNING_DAYS = 30

/** One secret in a list: name, environment, expiry warning and a copy button. */
export function SecretRow({ secret, selected, onSelect, onCopy }: SecretRowProps) {
  const expiry = describeExpiry(secret.expires_in_days)
  const expiringSoon =
    secret.expires_in_days !== null && secret.expires_in_days <= EXPIRY_WARNING_DAYS
  return (
    <li
      className={cn(
        'flex items-center gap-3 rounded-lg border p-1 pr-3 transition-colors',
        selected ? 'border-info bg-info/10' : 'border-border bg-card hover:bg-muted/50',
      )}
    >
      <button
        type="button"
        onClick={onSelect}
        aria-pressed={selected}
        className="flex min-w-0 flex-1 items-center gap-3.5 rounded-md p-3 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted">
          <KeyRound className="size-[18px]" />
        </span>
        <span className="flex min-w-0 flex-col gap-0.5">
          <span className="truncate font-mono text-sm font-medium">{secret.name}</span>
          <span className="truncate text-[13px] text-muted-foreground">
            {`updated ${timeAgo(secret.updated_at)}`}
          </span>
        </span>
      </button>
      {expiry && expiringSoon ? (
        <span className="hidden text-xs font-medium text-warning lg:inline">{expiry}</span>
      ) : null}
      {secret.favorite ? (
        <Star className="size-4 fill-warning text-warning" aria-label="Favorite" />
      ) : null}
      {secret.environment ? <EnvBadge name={secret.environment} /> : null}
      <Button variant="ghost" size="icon-sm" aria-label={`Copy ${secret.name}`} onClick={onCopy}>
        <Copy />
      </Button>
    </li>
  )
}
