import { useNavigate } from '@tanstack/react-router'
import { Copy, Eye, EyeOff, History, KeyRound, Pencil, Star, Trash2, X } from 'lucide-react'
import { useEffect, useState } from 'react'

import { EnvBadge } from '@/components/general/env-badge'
import { ErrorNotice } from '@/components/general/error-notice'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useCopySecret } from '@/hooks/use-clipboard'
import { useRequireUnlock } from '@/hooks/use-require-unlock'
import { useRevealSecret, useSecret, useUpdateSecret } from '@/hooks/use-secrets'
import { useSettings } from '@/hooks/use-settings'
import { describeExpiry, formatDate, timeAgo } from '@/lib/format'
import type { SecretDetail } from '@/types'

import type { useVaultNav } from './use-vault-nav'

interface PanelProps {
  vaultId: string
  secretId: string
  nav: ReturnType<typeof useVaultNav>
}

/** Details of the selected secret: masked value, metadata and actions. */
export function SecretDetailPanel({ vaultId, secretId, nav }: PanelProps) {
  const { data: secret, error, isLoading } = useSecret(vaultId, secretId)
  return (
    <aside
      className="flex w-[420px] shrink-0 flex-col gap-5 overflow-y-auto border-l border-border p-6"
      aria-label="Secret details"
    >
      {isLoading ? <Skeleton className="h-64 w-full" /> : null}
      <ErrorNotice error={error} />
      {secret ? <Content vaultId={vaultId} secret={secret} nav={nav} /> : null}
    </aside>
  )
}

function Content({
  vaultId,
  secret,
  nav,
}: {
  vaultId: string
  secret: SecretDetail
  nav: PanelProps['nav']
}) {
  const update = useUpdateSecret(vaultId)
  const navigate = useNavigate()
  const requireUnlock = useRequireUnlock(vaultId)
  const toggleFavorite = () =>
    requireUnlock(() =>
      update.mutate({
        vault_id: vaultId,
        secret_id: secret.id,
        name: secret.name,
        kind: secret.kind,
        value: null,
        environment: secret.environment,
        tags: secret.tags,
        notes: secret.notes,
        expires_at: secret.expires_at,
        remind_before_days: secret.remind_before_days,
        favorite: !secret.favorite,
      }),
    )
  return (
    <>
      <header className="flex items-center gap-3.5">
        <span className="flex size-11 items-center justify-center rounded-xl bg-muted">
          <KeyRound className="size-[22px]" />
        </span>
        <div className="min-w-0 flex-1">
          <h2 className="truncate font-mono text-[17px] font-semibold">{secret.name}</h2>
          <p className="text-[13px] text-muted-foreground">{secret.kind.replace('_', ' ')}</p>
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={secret.favorite ? 'Remove from favorites' : 'Add to favorites'}
          onClick={toggleFavorite}
        >
          <Star className={secret.favorite ? 'fill-warning text-warning' : ''} />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Close details"
          onClick={nav.clearSelection}
        >
          <X />
        </Button>
      </header>
      <ValueBox key={secret.id} vaultId={vaultId} secret={secret} />
      <Details secret={secret} />
      <div className="flex gap-2.5">
        <Button variant="secondary" onClick={() => requireUnlock(() => nav.openPanel('edit'))}>
          <Pencil /> Edit
        </Button>
        <Button
          variant="secondary"
          onClick={() =>
            requireUnlock(
              () =>
                void navigate({
                  to: '/vaults/$vaultId/history/$secretId',
                  params: { vaultId, secretId: secret.id },
                }),
            )
          }
        >
          <History /> History
        </Button>
        <Button variant="danger" onClick={() => requireUnlock(() => nav.openPanel('delete'))}>
          <Trash2 /> Delete
        </Button>
      </div>
    </>
  )
}

function ValueBox({ vaultId, secret }: { vaultId: string; secret: SecretDetail }) {
  const reveal = useRevealSecret(vaultId)
  const copy = useCopySecret(vaultId)
  const { data: settings } = useSettings()
  const [value, setValue] = useState<string | null>(null)
  const requireUnlock = useRequireUnlock(vaultId)

  useEffect(() => {
    const seconds = settings?.hide_revealed_seconds ?? 0
    if (value === null || seconds === 0) return
    const timer = setTimeout(() => setValue(null), seconds * 1000)
    return () => clearTimeout(timer)
  }, [value, settings?.hide_revealed_seconds])

  const toggle = () => {
    if (value !== null) return setValue(null)
    requireUnlock(() => reveal.mutate(secret.id, { onSuccess: (r) => setValue(r.value) }))
  }

  return (
    <section className="flex flex-col gap-2">
      <h3 className="text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
        Value
      </h3>
      <div className="flex items-center gap-2.5 rounded-md border border-input bg-inset px-3 py-3">
        <code className="min-w-0 flex-1 break-all font-mono text-[13px]" data-testid="secret-value">
          {value ?? '••••••••••••••••'}
        </code>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={value === null ? 'Reveal value' : 'Hide value'}
          onClick={toggle}
        >
          {value === null ? <Eye /> : <EyeOff />}
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Copy value"
          onClick={() => requireUnlock(() => copy.mutate(secret.id))}
        >
          <Copy />
        </Button>
      </div>
      <ErrorNotice error={reveal.error} />
    </section>
  )
}

function Details({ secret }: { secret: SecretDetail }) {
  const expiry = describeExpiry(secret.expires_in_days)
  const rows: [string, React.ReactNode][] = [
    ['Environment', secret.environment ? <EnvBadge key="e" name={secret.environment} /> : '-'],
    ['Created', formatDate(secret.created_at)],
    ['Updated', `${timeAgo(secret.updated_at)} · v${secret.version}`],
    ['Expires', expiry ?? 'Never'],
    ['Tags', secret.tags.length ? secret.tags.join(', ') : '-'],
  ]
  return (
    <section className="flex flex-col gap-1">
      <h3 className="pb-1 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
        Details
      </h3>
      <dl>
        {rows.map(([label, value]) => (
          <div
            key={label}
            className="flex items-center gap-3 border-b border-border py-2.5 text-[13px]"
          >
            <dt className="w-28 shrink-0 text-muted-foreground">{label}</dt>
            <dd className="min-w-0 flex-1 truncate">{value}</dd>
          </div>
        ))}
      </dl>
      {secret.notes ? (
        <p className="pt-2 text-[13px] whitespace-pre-wrap text-muted-foreground">{secret.notes}</p>
      ) : null}
    </section>
  )
}
